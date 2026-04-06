//! Telemetry ingestion: CCSDS primary header + CFE ES HK payload parsing (Linux LE).

pub mod cfe_primary;
pub mod es_hk;

#[cfg(feature = "server")]
pub mod udp_task;

use serde::Serialize;

use crate::tlm::cfe_primary::CcsdsPrimaryHeader;
use crate::tlm::es_hk::{parse_es_hk_datagram, EsHkV1};

/// Wire JSON for WebSocket clients (`kind` discriminates schema).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TlmEvent {
    EsHkV1 {
        received_at: String,
        raw_len: usize,
        primary: CcsdsPrimarySummary,
        es_hk: EsHkV1,
    },
    ParseError {
        received_at: String,
        raw_len: usize,
        primary: Option<CcsdsPrimarySummary>,
        message: String,
        hex_preview: String,
    },
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct CcsdsPrimarySummary {
    pub apid: u16,
    pub packet_type: u8,
    pub sequence_count: u16,
}

impl From<&CcsdsPrimaryHeader> for CcsdsPrimarySummary {
    fn from(h: &CcsdsPrimaryHeader) -> Self {
        Self {
            apid: h.apid,
            packet_type: h.packet_type,
            sequence_count: h.sequence_count,
        }
    }
}

/// Classify and parse a raw UDP datagram from TO_LAB / mock.
pub fn classify_datagram(data: &[u8], received_at: String) -> TlmEvent {
    let raw_len = data.len();
    let primary = CcsdsPrimaryHeader::parse(data);
    let primary_summary = primary.as_ref().map(CcsdsPrimarySummary::from);

    if let Some(ref ph) = primary {
        let expected = ph.total_bytes_including_primary();
        if data.len() != expected {
            return TlmEvent::ParseError {
                received_at,
                raw_len,
                primary: primary_summary,
                message: format!("length mismatch: header implies {expected} bytes, got {raw_len}"),
                hex_preview: hex_preview(data),
            };
        }
    }

    if let Some(hk) = parse_es_hk_datagram(data) {
        let primary = primary_summary.unwrap_or(CcsdsPrimarySummary {
            apid: 0,
            packet_type: 0,
            sequence_count: 0,
        });
        return TlmEvent::EsHkV1 {
            received_at,
            raw_len,
            primary,
            es_hk: hk,
        };
    }

    TlmEvent::ParseError {
        received_at,
        raw_len,
        primary: primary_summary,
        message: "not a recognized ES HK datagram (expected 12 + 168 bytes for ES HK v1)".into(),
        hex_preview: hex_preview(data),
    }
}

fn hex_preview(data: &[u8]) -> String {
    let take = data.len().min(64);
    let mut s = String::with_capacity(take * 3);
    for (i, b) in data.iter().take(take).enumerate() {
        if i > 0 {
            s.push(' ');
        }
        s.push_str(&format!("{b:02x}"));
    }
    if data.len() > take {
        s.push_str(" …");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tlm::es_hk::{CFE_TLM_HEADER_PREFIX_BYTES, ES_HK_PAYLOAD_BYTES};

    #[test]
    fn classify_es_hk_round_trip() {
        let total = CFE_TLM_HEADER_PREFIX_BYTES + ES_HK_PAYLOAD_BYTES;
        let mut d = vec![0u8; total];
        // TM (type 0), secondary header flag, APID 0; unsegmented seq 0; data field = total - 6
        let user_len = total - 6;
        let w2 = (user_len - 1) as u16;
        d[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
        d[2..4].copy_from_slice(&0xc000u16.to_be_bytes());
        d[4..6].copy_from_slice(&w2.to_be_bytes());
        d[CFE_TLM_HEADER_PREFIX_BYTES] = 0xAB;

        let ev = classify_datagram(&d, "test".into());
        match ev {
            TlmEvent::EsHkV1 { es_hk, .. } => assert_eq!(es_hk.command_counter, 0xAB),
            TlmEvent::ParseError { message, .. } => panic!("unexpected error: {message}"),
        }
    }
}
