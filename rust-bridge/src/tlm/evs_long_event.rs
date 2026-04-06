//! EVS long event telemetry parsing (cFE EVS LONG_EVENT_MSG).
//!
//! Layout is derived from `cfe_evs.xml` EDS and sample mission config:
//! - ApiName length: 20 (`CFE_MISSION_MAX_API_LEN`)
//! - Event message length: 122 (`CFE_MISSION_EVS_MAX_MESSAGE_LENGTH`)
//! - Telemetry prefix: 6-byte CCSDS primary + 2-byte MsgId + 8-byte time = 16 bytes

use serde::Serialize;

pub const EVS_LONG_EVENT_APID_LEGACY: u16 = 0x009;
pub const EVS_LONG_EVENT_MSGID_LE_LEGACY: u16 = 0x0809;

pub const API_NAME_BYTES: usize = 20;
pub const EVENT_MESSAGE_BYTES: usize = 122;
pub const CFE_TLM_HEADER_PREFIX_BYTES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvsPacketIdV1 {
    pub app_name: String,
    pub event_id: u16,
    pub event_type: u16,
    pub spacecraft_id: u32,
    pub processor_id: u32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EvsLongEventV1 {
    pub packet_id: EvsPacketIdV1,
    pub message: String,
}

fn decode_c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    let slice = &bytes[..end];
    let s = String::from_utf8_lossy(slice).to_string();
    s.trim_end_matches(|c: char| c.is_whitespace()).to_string()
}

fn looks_like_printable_ascii(bytes: &[u8]) -> bool {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    if end == 0 {
        return false;
    }
    bytes[..end]
        .iter()
        .all(|&b| (0x20..=0x7Eu8).contains(&b))
}

/// Parses a full UDP datagram when it carries an EVS LONG_EVENT_MSG payload.
pub fn parse_evs_long_event_datagram(data: &[u8]) -> Option<EvsLongEventV1> {
    let ph = crate::tlm::cfe_primary::CcsdsPrimaryHeader::parse(data)?;
    if !ph.secondary_header_flag {
        return None;
    }
    if data.len() != ph.total_bytes_including_primary() {
        return None;
    }

    // Legacy heuristic: APID 9 for EVS long event topic id (TELEMETRY_BASE_TOPICID + 9).
    // For EDS missions, the MsgId at bytes 6–7 may differ; we also accept packets that
    // strongly resemble the EVS long event layout (printable ApiName + message).
    let msg_id = if data.len() >= 8 {
        u16::from_le_bytes([data[6], data[7]])
    } else {
        return None;
    };
    let legacy_match = ph.apid == EVS_LONG_EVENT_APID_LEGACY && msg_id == EVS_LONG_EVENT_MSGID_LE_LEGACY;

    let need = CFE_TLM_HEADER_PREFIX_BYTES + API_NAME_BYTES + 2 + 2 + 4 + 4 + EVENT_MESSAGE_BYTES;
    if data.len() < need {
        return None;
    }

    let off = CFE_TLM_HEADER_PREFIX_BYTES;
    let app_name_raw = &data[off..off + API_NAME_BYTES];
    if !legacy_match && !looks_like_printable_ascii(app_name_raw) {
        return None;
    }
    let app_name = decode_c_string(app_name_raw);

    let mut i = off + API_NAME_BYTES;
    let event_id = u16::from_le_bytes([data[i], data[i + 1]]);
    i += 2;
    let event_type = u16::from_le_bytes([data[i], data[i + 1]]);
    i += 2;
    let spacecraft_id = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
    i += 4;
    let processor_id = u32::from_le_bytes([data[i], data[i + 1], data[i + 2], data[i + 3]]);
    i += 4;

    let msg_raw = &data[i..i + EVENT_MESSAGE_BYTES];
    if !legacy_match && !looks_like_printable_ascii(msg_raw) {
        return None;
    }
    let message = decode_c_string(msg_raw);

    Some(EvsLongEventV1 {
        packet_id: EvsPacketIdV1 {
            app_name,
            event_id,
            event_type,
            spacecraft_id,
            processor_id,
        },
        message,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_synthetic_legacy_packet() {
        // Build a synthetic packet: CCSDS primary (APID 9), MsgId LE 0x0809, 8-byte time, then payload.
        let total = CFE_TLM_HEADER_PREFIX_BYTES + API_NAME_BYTES + 2 + 2 + 4 + 4 + EVENT_MESSAGE_BYTES;
        let user = (total - 6) as u16;
        let w2 = user - 1;
        let mut d = vec![0u8; total];
        d[0..2].copy_from_slice(&(0x0800u16 | EVS_LONG_EVENT_APID_LEGACY).to_be_bytes());
        d[2..4].copy_from_slice(&0xC000u16.to_be_bytes());
        d[4..6].copy_from_slice(&w2.to_be_bytes());
        d[6..8].copy_from_slice(&EVS_LONG_EVENT_MSGID_LE_LEGACY.to_le_bytes());
        // time bytes [8..16] left as 0

        let off = CFE_TLM_HEADER_PREFIX_BYTES;
        let app = b"CFE_EVS\0";
        d[off..off + app.len()].copy_from_slice(app);
        let mut i = off + API_NAME_BYTES;
        d[i..i + 2].copy_from_slice(&123u16.to_le_bytes());
        i += 2;
        d[i..i + 2].copy_from_slice(&2u16.to_le_bytes());
        i += 2;
        d[i..i + 4].copy_from_slice(&66u32.to_le_bytes());
        i += 4;
        d[i..i + 4].copy_from_slice(&1u32.to_le_bytes());
        i += 4;
        let msg = b"hello world\0";
        d[i..i + msg.len()].copy_from_slice(msg);

        let ev = parse_evs_long_event_datagram(&d).expect("parse");
        assert_eq!(ev.packet_id.app_name, "CFE_EVS");
        assert_eq!(ev.packet_id.event_id, 123);
        assert_eq!(ev.packet_id.event_type, 2);
        assert_eq!(ev.packet_id.spacecraft_id, 66);
        assert_eq!(ev.packet_id.processor_id, 1);
        assert_eq!(ev.message, "hello world");
    }
}

