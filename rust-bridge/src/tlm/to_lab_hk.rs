//! TO_LAB housekeeping telemetry — small payload after the standard cFE TLM prefix.
//!
//! Classification uses LE MsgId `0x0880` at bytes 6–7 (after the 6-byte CCSDS primary), matching
//! `TO_LAB_HK_TLM_MID` for the bundled mission (`0x0800 | 0x80`). If your EDS wire layout differs,
//! adjust [`parse_to_lab_hk_datagram`] after capturing a live datagram.

use serde::Serialize;

use crate::tlm::es_hk::CFE_TLM_HEADER_PREFIX_BYTES;

/// `TO_LAB_HK_TLM` MsgId value for the default cpu1 mission (see `default_to_lab_msgids.h`).
pub const TO_LAB_HK_TLM_MSGID_LE: u16 = 0x0880;

/// Housekeeping payload only (after the 12-byte CCSDS + cFE telemetry secondary header).
pub const TO_LAB_HK_PAYLOAD_BYTES: usize = 4;

/// Parsed TO_LAB HK fields used in the UI.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToLabHkV1 {
    pub command_counter: u8,
    pub command_error_counter: u8,
}

/// Parses a full UDP datagram when it is **not** ES HK and carries `TO_LAB_HK_TLM` MsgId.
pub fn parse_to_lab_hk_datagram(data: &[u8]) -> Option<ToLabHkV1> {
    let ph = crate::tlm::cfe_primary::CcsdsPrimaryHeader::parse(data)?;
    if !ph.secondary_header_flag {
        return None;
    }
    if data.len() < CFE_TLM_HEADER_PREFIX_BYTES + TO_LAB_HK_PAYLOAD_BYTES {
        return None;
    }
    if data.len() != ph.total_bytes_including_primary() {
        return None;
    }
    let msg_id = u16::from_le_bytes([data[6], data[7]]);
    if msg_id != TO_LAB_HK_TLM_MSGID_LE {
        return None;
    }
    let off = CFE_TLM_HEADER_PREFIX_BYTES;
    Some(ToLabHkV1 {
        command_counter: data[off],
        command_error_counter: data[off + 1],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_min_to_lab_hk_packet() -> Vec<u8> {
        let user = 16u16;
        let w2 = user - 1;
        let mut d = vec![0u8; 6 + user as usize];
        d[0..2].copy_from_slice(&0x0800u16.to_be_bytes());
        d[2..4].copy_from_slice(&0xC000u16.to_be_bytes());
        d[4..6].copy_from_slice(&w2.to_be_bytes());
        d[6..8].copy_from_slice(&TO_LAB_HK_TLM_MSGID_LE.to_le_bytes());
        d[8..12].copy_from_slice(&[0u8; 4]);
        d[12] = 7;
        d[13] = 2;
        d
    }

    #[test]
    fn parse_round_trip_synthetic() {
        let d = build_min_to_lab_hk_packet();
        let hk = parse_to_lab_hk_datagram(&d).expect("parse");
        assert_eq!(hk.command_counter, 7);
        assert_eq!(hk.command_error_counter, 2);
    }
}
