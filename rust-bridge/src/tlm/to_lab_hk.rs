//! TO_LAB housekeeping telemetry — small payload after the standard cFE TLM prefix.
//!
//! Classification uses the verified on-wire MsgId in the cFE telemetry secondary header at bytes 6–7
//! (after the 6-byte CCSDS primary header). In EDS deployments, the topic→MsgId mapping is runtime
//! (`CFE_SB_LocalTlmTopicIdToMsgId`), so the MsgId may differ from the simple `0x0800 | topic` mapping.

use serde::Serialize;

use crate::tlm::es_hk::CFE_TLM_HEADER_PREFIX_BYTES;

/// Verified on-wire `TO_LAB_HK_TLM` MsgId (little-endian u16 at bytes 6–7) for the sample mission E2E capture.
pub const TO_LAB_HK_TLM_MSGID_LE_EDS: u16 = 0x0F00;

/// Legacy/non-EDS `TO_LAB_HK_TLM` MsgId value for the default cpu1 mission (`0x0800 | 0x80`).
pub const TO_LAB_HK_TLM_MSGID_LE_LEGACY: u16 = 0x0880;

/// Verified CCSDS APID for the 20-byte TO_LAB HK datagram capture.
pub const TO_LAB_HK_APID: u16 = 0x080;

/// Housekeeping payload only (after the 12-byte CCSDS + cFE telemetry secondary header).
pub const TO_LAB_HK_PAYLOAD_BYTES: usize = 4;

/// Bytes before the HK payload in the verified 20-byte capture:
/// 6-byte CCSDS primary + 2-byte MsgId + 8-byte cFE time.
pub const TO_LAB_HK_HEADER_PREFIX_BYTES_CAPTURE_V1: usize = 16;

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
    if ph.apid != TO_LAB_HK_APID {
        return None;
    }
    let msg_id = u16::from_le_bytes([data[6], data[7]]);
    if msg_id != TO_LAB_HK_TLM_MSGID_LE_EDS && msg_id != TO_LAB_HK_TLM_MSGID_LE_LEGACY {
        return None;
    }
    // The verified on-wire 20-byte packet includes an 8-byte cFE time field after MsgId.
    // For now, treat that capture as the stability target.
    let off = TO_LAB_HK_HEADER_PREFIX_BYTES_CAPTURE_V1;
    Some(ToLabHkV1 {
        command_counter: data[off],
        command_error_counter: data[off + 1],
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_min_to_lab_hk_packet() -> Vec<u8> {
        // Match the verified capture shape: 20 bytes total, CCSDS length field 0x000D.
        let user = 14u16;
        let w2 = user - 1;
        let mut d = vec![0u8; 6 + user as usize];
        d[0..2].copy_from_slice(&(0x0800u16 | TO_LAB_HK_APID).to_be_bytes());
        d[2..4].copy_from_slice(&0xC000u16.to_be_bytes());
        d[4..6].copy_from_slice(&w2.to_be_bytes());
        d[6..8].copy_from_slice(&TO_LAB_HK_TLM_MSGID_LE_EDS.to_le_bytes());
        // 8-byte cFE time field placeholder
        d[8..16].copy_from_slice(&[0u8; 8]);
        // Payload (4 bytes) starts at byte 16 in the verified capture.
        d[16] = 7;
        d[17] = 2;
        d
    }

    fn build_min_to_lab_hk_packet_legacy_msgid() -> Vec<u8> {
        let mut d = build_min_to_lab_hk_packet();
        d[6..8].copy_from_slice(&TO_LAB_HK_TLM_MSGID_LE_LEGACY.to_le_bytes());
        d
    }

    #[test]
    fn parse_round_trip_legacy_synthetic() {
        let d = build_min_to_lab_hk_packet_legacy_msgid();
        let hk = parse_to_lab_hk_datagram(&d).expect("parse");
        assert_eq!(hk.command_counter, 7);
        assert_eq!(hk.command_error_counter, 2);
    }

    #[test]
    fn parse_round_trip_synthetic() {
        let d = build_min_to_lab_hk_packet();
        let hk = parse_to_lab_hk_datagram(&d).expect("parse");
        assert_eq!(hk.command_counter, 7);
        assert_eq!(hk.command_error_counter, 2);
    }

    #[test]
    fn parse_verified_20_byte_capture() {
        // From docs/AVAILABLE_TELEMETRY.md (20-byte datagram, APID 128, length field 0x000D).
        let d: [u8; 20] = [
            0x08, 0x80, 0xc0, 0xa5, 0x00, 0x0d, 0x00, 0x0f, 0x46, 0xcd, 0xb1, 0x16, 0x00, 0x00,
            0x00, 0x00, 0x01, 0x00, 0x00, 0x00,
        ];
        let hk = parse_to_lab_hk_datagram(&d).expect("parse");
        assert_eq!(hk.command_counter, 0x01);
        assert_eq!(hk.command_error_counter, 0x00);
    }
}
