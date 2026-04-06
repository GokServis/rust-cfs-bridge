//! `CFE_ES_HousekeepingTlm_Payload` for mission `CFE_MISSION_ES_PERF_MAX_IDS == 128` (Linux LE).

use serde::Serialize;

/// Must match [sample mission](cfs/cmake/sample_defs/example_mission_cfg.h) default.
pub const CFE_MISSION_ES_PERF_MAX_IDS: usize = 128;

const _: () = assert!(CFE_MISSION_ES_PERF_MAX_IDS / 32 == 4);

/// Size of the HK payload only (after telemetry secondary header).
pub const ES_HK_PAYLOAD_BYTES: usize = 168;

/// Bytes before HK payload: 6-byte CCSDS primary + 6-byte cFE telemetry secondary (see cfe-es-hk-tlm.txt offsets).
pub const CFE_TLM_HEADER_PREFIX_BYTES: usize = 12;

/// Executive Services HK fields used by the dashboard (native-endian on Linux sim).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EsHkV1 {
    pub command_counter: u8,
    pub command_error_counter: u8,
    pub cfe_core_checksum: u16,
    pub cfe_version: [u8; 4],
    pub osal_version: [u8; 4],
    pub psp_version: [u8; 4],
    pub syslog_bytes_used: u64,
    pub syslog_size: u64,
    pub syslog_entries: u32,
    pub syslog_mode: u32,
    pub registered_core_apps: u32,
    pub registered_external_apps: u32,
    pub registered_tasks: u32,
    pub registered_libs: u32,
    pub reset_type: u32,
    pub reset_subtype: u32,
    pub processor_resets: u32,
    pub max_processor_resets: u32,
    pub boot_source: u32,
    pub perf_state: u32,
    pub perf_mode: u32,
    pub perf_trigger_count: u32,
    pub heap_bytes_free: u64,
    pub heap_blocks_free: u64,
    pub heap_max_block_size: u64,
}

fn read_u32_le(b: &[u8], o: usize) -> Option<u32> {
    b.get(o..o + 4)
        .map(|s| u32::from_le_bytes(s.try_into().unwrap()))
}

fn read_u64_le(b: &[u8], o: usize) -> Option<u64> {
    b.get(o..o + 8)
        .map(|s| u64::from_le_bytes(s.try_into().unwrap()))
}

/// Parses ES HK payload bytes (starts at Command Counter). `b` must be exactly [`ES_HK_PAYLOAD_BYTES`] for full parse.
pub fn parse_es_hk_payload(b: &[u8]) -> Option<EsHkV1> {
    if b.len() < ES_HK_PAYLOAD_BYTES {
        return None;
    }
    let cfe_version = [b[4], b[5], b[6], b[7]];
    let osal_version = [b[8], b[9], b[10], b[11]];
    let psp_version = [b[12], b[13], b[14], b[15]];

    Some(EsHkV1 {
        command_counter: b[0],
        command_error_counter: b[1],
        cfe_core_checksum: u16::from_le_bytes([b[2], b[3]]),
        cfe_version,
        osal_version,
        psp_version,
        syslog_bytes_used: read_u64_le(b, 16)?,
        syslog_size: read_u64_le(b, 24)?,
        syslog_entries: read_u32_le(b, 32)?,
        syslog_mode: read_u32_le(b, 36)?,
        registered_core_apps: read_u32_le(b, 48)?,
        registered_external_apps: read_u32_le(b, 52)?,
        registered_tasks: read_u32_le(b, 56)?,
        registered_libs: read_u32_le(b, 60)?,
        reset_type: read_u32_le(b, 64)?,
        reset_subtype: read_u32_le(b, 68)?,
        processor_resets: read_u32_le(b, 72)?,
        max_processor_resets: read_u32_le(b, 76)?,
        boot_source: read_u32_le(b, 80)?,
        perf_state: read_u32_le(b, 84)?,
        perf_mode: read_u32_le(b, 88)?,
        perf_trigger_count: read_u32_le(b, 92)?,
        heap_bytes_free: read_u64_le(b, 144)?,
        heap_blocks_free: read_u64_le(b, 152)?,
        heap_max_block_size: read_u64_le(b, 160)?,
    })
}

/// Full UDP datagram: 12-byte headers + [`ES_HK_PAYLOAD_BYTES`] HK payload.
pub fn parse_es_hk_datagram(data: &[u8]) -> Option<EsHkV1> {
    if data.len() < CFE_TLM_HEADER_PREFIX_BYTES + ES_HK_PAYLOAD_BYTES {
        return None;
    }
    parse_es_hk_payload(&data[CFE_TLM_HEADER_PREFIX_BYTES..])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn golden_payload() -> Vec<u8> {
        let mut v = vec![0u8; ES_HK_PAYLOAD_BYTES];
        v[0] = 1;
        v[1] = 2;
        v[2] = 0x34;
        v[3] = 0x12;
        v[48] = 3;
        v[49] = 0;
        v[50] = 0;
        v[51] = 0;
        v[144] = 0x40;
        v[145] = 0x42;
        v[146] = 0;
        v[147] = 0;
        v[148] = 0;
        v[149] = 0;
        v[150] = 0;
        v[151] = 0;
        v
    }

    #[test]
    fn parse_round_trip_golden() {
        let p = golden_payload();
        let h = parse_es_hk_payload(&p).expect("parse");
        assert_eq!(h.command_counter, 1);
        assert_eq!(h.command_error_counter, 2);
        assert_eq!(h.cfe_core_checksum, 0x1234);
        assert_eq!(h.registered_core_apps, 3);
        assert_eq!(h.heap_bytes_free, 0x4240);
    }

    #[test]
    fn parse_datagram_with_prefix() {
        let mut d = vec![0xFFu8; CFE_TLM_HEADER_PREFIX_BYTES];
        d.extend_from_slice(&golden_payload());
        let h = parse_es_hk_datagram(&d).expect("datagram");
        assert_eq!(h.command_counter, 1);
    }

    #[test]
    fn short_buffer_returns_none() {
        assert!(parse_es_hk_payload(&[0u8; 10]).is_none());
    }
}
