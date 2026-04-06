//! Telemetry ingestion: CCSDS primary header + CFE ES HK payload parsing (Linux LE).

pub mod cfe_primary;
pub mod es_hk;
pub mod evs_long_event;
pub mod to_lab_hk;

#[cfg(feature = "server")]
pub mod udp_task;

use serde::Serialize;

use crate::tlm::cfe_primary::CcsdsPrimaryHeader;
use crate::tlm::es_hk::{parse_es_hk_datagram, EsHkV1};
use crate::tlm::evs_long_event::{parse_evs_long_event_datagram, EvsLongEventV1};
use crate::tlm::to_lab_hk::{parse_to_lab_hk_datagram, ToLabHkV1};

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
    ToLabHkV1 {
        received_at: String,
        raw_len: usize,
        primary: CcsdsPrimarySummary,
        to_lab_hk: ToLabHkV1,
    },
    EvsLongEventV1 {
        received_at: String,
        raw_len: usize,
        primary: CcsdsPrimarySummary,
        evs_long_event: EvsLongEventV1,
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

    if let Some(hk) = parse_to_lab_hk_datagram(data) {
        let primary = primary_summary.unwrap_or(CcsdsPrimarySummary {
            apid: 0,
            packet_type: 0,
            sequence_count: 0,
        });
        return TlmEvent::ToLabHkV1 {
            received_at,
            raw_len,
            primary,
            to_lab_hk: hk,
        };
    }

    if let Some(evs) = parse_evs_long_event_datagram(data) {
        let primary = primary_summary.unwrap_or(CcsdsPrimarySummary {
            apid: 0,
            packet_type: 0,
            sequence_count: 0,
        });
        return TlmEvent::EvsLongEventV1 {
            received_at,
            raw_len,
            primary,
            evs_long_event: evs,
        };
    }

    TlmEvent::ParseError {
        received_at,
        raw_len,
        primary: primary_summary,
        message: "not a recognized ES HK, TO_LAB HK, or EVS long event datagram".into(),
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
            TlmEvent::ToLabHkV1 { .. } => panic!("unexpected TO_LAB HK"),
            TlmEvent::EvsLongEventV1 { .. } => panic!("unexpected EVS long event"),
            TlmEvent::ParseError { message, .. } => panic!("unexpected error: {message}"),
        }
    }

    #[test]
    fn classify_length_mismatch_is_parse_error() {
        let mut d = vec![0u8; 32];
        d[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
        d[2..4].copy_from_slice(&0xc000u16.to_be_bytes());
        // data_length_field 0x001A => user data 27 bytes => total 33 bytes; buffer is only 32
        d[4..6].copy_from_slice(&0x001Au16.to_be_bytes());
        let ev = classify_datagram(&d, "t".into());
        match ev {
            TlmEvent::ParseError { message, .. } => assert!(message.contains("length mismatch")),
            TlmEvent::EsHkV1 { .. }
            | TlmEvent::ToLabHkV1 { .. }
            | TlmEvent::EvsLongEventV1 { .. } => {
                panic!("expected parse_error")
            }
        }
    }

    #[test]
    fn classify_garbage_short_buffer_is_parse_error() {
        let d = vec![0x01u8, 0x02, 0x03];
        let ev = classify_datagram(&d, "t".into());
        assert!(matches!(ev, TlmEvent::ParseError { .. }));
    }

    #[test]
    fn classify_evs_long_event_round_trip() {
        use crate::tlm::evs_long_event::{
            API_NAME_BYTES, CFE_TLM_HEADER_PREFIX_BYTES, EVENT_MESSAGE_BYTES,
            EVS_LONG_EVENT_APID_LEGACY, EVS_LONG_EVENT_MSGID_LE_LEGACY,
        };
        let total =
            CFE_TLM_HEADER_PREFIX_BYTES + API_NAME_BYTES + 2 + 2 + 4 + 4 + EVENT_MESSAGE_BYTES;
        let user = (total - 6) as u16;
        let w2 = user - 1;
        let mut d = vec![0u8; total];
        d[0..2].copy_from_slice(&(0x0800u16 | EVS_LONG_EVENT_APID_LEGACY).to_be_bytes());
        d[2..4].copy_from_slice(&0xC000u16.to_be_bytes());
        d[4..6].copy_from_slice(&w2.to_be_bytes());
        d[6..8].copy_from_slice(&EVS_LONG_EVENT_MSGID_LE_LEGACY.to_le_bytes());
        let off = CFE_TLM_HEADER_PREFIX_BYTES;
        d[off..off + 8].copy_from_slice(b"CFE_EVS\0");
        let mut i = off + API_NAME_BYTES;
        d[i..i + 2].copy_from_slice(&1u16.to_le_bytes());
        i += 2;
        d[i..i + 2].copy_from_slice(&2u16.to_le_bytes());
        i += 2;
        d[i..i + 4].copy_from_slice(&3u32.to_le_bytes());
        i += 4;
        d[i..i + 4].copy_from_slice(&4u32.to_le_bytes());
        i += 4;
        d[i..i + 5].copy_from_slice(b"hi\0\0\0");

        let ev = classify_datagram(&d, "test".into());
        match ev {
            TlmEvent::EvsLongEventV1 { evs_long_event, .. } => {
                assert_eq!(evs_long_event.packet_id.app_name, "CFE_EVS");
                assert_eq!(evs_long_event.message, "hi");
            }
            _ => panic!("expected EVS long event"),
        }
    }

    #[test]
    fn classify_parse_error_hex_preview_truncates_long_buffer() {
        let mut d = vec![0xAAu8; 100];
        d[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
        d[2..4].copy_from_slice(&0xc000u16.to_be_bytes());
        // user data 94 bytes => total 100
        d[4..6].copy_from_slice(&0x005Du16.to_be_bytes());
        let ev = classify_datagram(&d, "t".into());
        match ev {
            TlmEvent::ParseError { hex_preview, .. } => {
                assert!(hex_preview.contains(" …"));
            }
            _ => panic!("expected parse_error"),
        }
    }
}
