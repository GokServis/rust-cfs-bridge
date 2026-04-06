/** Mirrors `rust_bridge::tlm::TlmEvent` JSON (serde tag `kind`). */

export interface EsHkPrimary {
  apid: number
  packet_type: number
  sequence_count: number
}

export interface EsHkPayload {
  command_counter: number
  command_error_counter: number
  cfe_core_checksum: number
  cfe_version: number[]
  osal_version: number[]
  psp_version: number[]
  syslog_bytes_used: number
  syslog_size: number
  syslog_entries: number
  syslog_mode: number
  registered_core_apps: number
  registered_external_apps: number
  registered_tasks: number
  registered_libs: number
  reset_type: number
  reset_subtype: number
  processor_resets: number
  max_processor_resets: number
  boot_source: number
  perf_state: number
  perf_mode: number
  perf_trigger_count: number
  heap_bytes_free: number
  heap_blocks_free: number
  heap_max_block_size: number
}

export interface ToLabHkPayload {
  command_counter: number
  command_error_counter: number
}

export type TlmMessage =
  | {
      kind: 'es_hk_v1'
      received_at: string
      raw_len: number
      primary: EsHkPrimary
      es_hk: EsHkPayload
    }
  | {
      kind: 'to_lab_hk_v1'
      received_at: string
      raw_len: number
      primary: EsHkPrimary
      to_lab_hk: ToLabHkPayload
    }
  | {
      kind: 'parse_error'
      received_at: string
      raw_len: number
      primary: EsHkPrimary | null
      message: string
      hex_preview: string
    }
