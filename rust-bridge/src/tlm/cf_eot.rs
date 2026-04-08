//! CF end-of-transaction telemetry (`CF_EotPacket_Payload_t`) for lab gating (Linux LE).
//!
//! Source structs:
//! - `cfs/apps/cf/config/default_cf_msgstruct.h` (CF_EotPacket_t)
//! - `cfs/apps/cf/config/default_cf_msgdefs.h` (CF_EotPacket_Payload_t)

use serde::Serialize;

/// Bytes before payload: 6-byte CCSDS primary + 2-byte MsgId + 8-byte time.
pub const CFE_TLM_HEADER_PREFIX_BYTES: usize = 16;

/// CF filename max length in this mission (CF_FILENAME_MAX_LEN == CFE_MISSION_MAX_PATH_LEN == 64).
pub const CF_FILENAME_MAX_LEN: usize = 64;

/// Size of `CF_EotPacket_Payload_t` in this mission build.
pub const CF_EOT_PAYLOAD_BYTES: usize = 164;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CfEotV1 {
    pub seq_num: u32,
    pub channel: u32,
    pub direction: u32,
    pub state: u32,
    pub txn_stat: u32,
    pub src_eid: u32,
    pub peer_eid: u32,
    pub fsize: u32,
    pub crc_result: u32,
    pub src_filename: String,
    pub dst_filename: String,
}

fn read_u32_le(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4)
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

fn read_cstr(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).to_string()
}

pub fn parse_cf_eot_payload(b: &[u8]) -> Option<CfEotV1> {
    if b.len() < CF_EOT_PAYLOAD_BYTES {
        return None;
    }
    // Layout per `CF_EotPacket_Payload_t`:
    // u32 seq_num;
    // u32 channel, direction, state, txn_stat;
    // u32 src_eid, peer_eid;
    // u32 fsize, crc_result;
    // char src_filename[64], dst_filename[64];
    let seq_num = read_u32_le(b, 0)?;
    let channel = read_u32_le(b, 4)?;
    let direction = read_u32_le(b, 8)?;
    let state = read_u32_le(b, 12)?;
    let txn_stat = read_u32_le(b, 16)?;
    let src_eid = read_u32_le(b, 20)?;
    let peer_eid = read_u32_le(b, 24)?;
    let fsize = read_u32_le(b, 28)?;
    let crc_result = read_u32_le(b, 32)?;
    let src_filename = read_cstr(b.get(36..36 + CF_FILENAME_MAX_LEN)?);
    let dst_filename = read_cstr(b.get(36 + CF_FILENAME_MAX_LEN..36 + 2 * CF_FILENAME_MAX_LEN)?);

    Some(CfEotV1 {
        seq_num,
        channel,
        direction,
        state,
        txn_stat,
        src_eid,
        peer_eid,
        fsize,
        crc_result,
        src_filename,
        dst_filename,
    })
}

/// Full UDP datagram: 12-byte headers + [`CF_EOT_PAYLOAD_BYTES`] payload.
pub fn parse_cf_eot_datagram(data: &[u8]) -> Option<CfEotV1> {
    if data.len() < CFE_TLM_HEADER_PREFIX_BYTES + CF_EOT_PAYLOAD_BYTES {
        return None;
    }
    parse_cf_eot_payload(&data[CFE_TLM_HEADER_PREFIX_BYTES..])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_minimal_eot_payload() {
        let mut b = vec![0u8; CF_EOT_PAYLOAD_BYTES];
        b[0..4].copy_from_slice(&123u32.to_le_bytes()); // seq
        b[4..8].copy_from_slice(&0u32.to_le_bytes()); // channel
        b[16..20].copy_from_slice(&0u32.to_le_bytes()); // txn_stat
                                                        // filenames
        b[36..36 + 4].copy_from_slice(b"src\0");
        b[36 + CF_FILENAME_MAX_LEN..36 + CF_FILENAME_MAX_LEN + 5].copy_from_slice(b"/dst\0");

        let e = parse_cf_eot_payload(&b).expect("parse");
        assert_eq!(e.seq_num, 123);
        assert_eq!(e.src_filename, "src");
        assert_eq!(e.dst_filename, "/dst");
    }
}
