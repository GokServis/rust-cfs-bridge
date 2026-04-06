//! CCSDS Space Packet primary header (6 bytes), no CRC (telemetry downlink from cFS).

/// Parsed CCSDS primary header fields (telecommand or telemetry).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CcsdsPrimaryHeader {
    pub version: u8,
    pub packet_type: u8,
    pub secondary_header_flag: bool,
    pub apid: u16,
    pub sequence_flags: u8,
    pub sequence_count: u16,
    /// CCSDS packet data length field (last 16 bits of primary header).
    pub data_length_field: u16,
}

impl CcsdsPrimaryHeader {
    /// Parses the first 6 bytes as a CCSDS primary header (big-endian 16-bit words).
    pub fn parse(data: &[u8]) -> Option<Self> {
        if data.len() < 6 {
            return None;
        }
        let w0 = u16::from_be_bytes([data[0], data[1]]);
        let w1 = u16::from_be_bytes([data[2], data[3]]);
        let w2 = u16::from_be_bytes([data[4], data[5]]);

        Some(Self {
            version: ((w0 >> 13) & 0x7) as u8,
            packet_type: ((w0 >> 12) & 1) as u8,
            secondary_header_flag: ((w0 >> 11) & 1) != 0,
            apid: w0 & 0x7FF,
            sequence_flags: ((w1 >> 14) & 0x3) as u8,
            sequence_count: w1 & 0x3FFF,
            data_length_field: w2,
        })
    }

    /// Total packet length in bytes from primary header through user data field (inclusive), excluding CRC if present.
    /// `total_user_data = data_length_field + 1` per CCSDS; primary is 6 bytes.
    pub fn total_bytes_including_primary(&self) -> usize {
        let user_data_len = (self.data_length_field as usize).saturating_add(1);
        6 + user_data_len
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_matches_uplink_style_header() {
        let h = [0x18, 0x06, 0xc0, 0x00, 0x00, 0x02];
        let p = CcsdsPrimaryHeader::parse(&h).expect("parse");
        assert_eq!(p.apid, 0x006);
        assert_eq!(p.packet_type, 1);
    }
}
