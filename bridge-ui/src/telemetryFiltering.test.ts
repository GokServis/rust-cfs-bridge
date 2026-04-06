import { describe, expect, it } from 'vitest'

import {
  apidOf,
  filterEntries,
  matchesSearch,
  pageSlice,
  summaryLine,
  totalPageCount,
  type TlmEntry,
} from './telemetryFiltering'

const esMsg = {
  kind: 'es_hk_v1' as const,
  received_at: '2026-01-01T00:00:00Z',
  raw_len: 180,
  primary: { apid: 7, packet_type: 0, sequence_count: 0 },
  es_hk: {
    command_counter: 1,
    command_error_counter: 0,
    cfe_core_checksum: 0,
    cfe_version: [0, 0, 0, 0],
    osal_version: [0, 0, 0, 0],
    psp_version: [0, 0, 0, 0],
    syslog_bytes_used: 0,
    syslog_size: 0,
    syslog_entries: 0,
    syslog_mode: 0,
    registered_core_apps: 0,
    registered_external_apps: 0,
    registered_tasks: 0,
    registered_libs: 0,
    reset_type: 0,
    reset_subtype: 0,
    processor_resets: 0,
    max_processor_resets: 0,
    boot_source: 0,
    perf_state: 0,
    perf_mode: 0,
    perf_trigger_count: 0,
    heap_bytes_free: 100,
    heap_blocks_free: 0,
    heap_max_block_size: 0,
  },
}

const toLabMsg = {
  kind: 'to_lab_hk_v1' as const,
  received_at: '2026-01-01T00:00:00Z',
  raw_len: 22,
  primary: { apid: 0, packet_type: 0, sequence_count: 0 },
  to_lab_hk: { command_counter: 3, command_error_counter: 1 },
}

const parseErr = {
  kind: 'parse_error' as const,
  received_at: '2026-01-02T00:00:00Z',
  raw_len: 10,
  primary: { apid: 9, packet_type: 0, sequence_count: 0 },
  message: 'bad',
  hex_preview: 'aa bb',
}

const evsLong = {
  kind: 'evs_long_event_v1' as const,
  received_at: '2026-01-03T00:00:00Z',
  raw_len: 170,
  primary: { apid: 9, packet_type: 0, sequence_count: 0 },
  evs_long_event: {
    packet_id: {
      app_name: 'CFE_EVS',
      event_id: 123,
      event_type: 2,
      spacecraft_id: 66,
      processor_id: 1,
    },
    message: 'hello',
  },
}

describe('telemetryFiltering', () => {
  it('apidOf reads primary', () => {
    expect(apidOf(esMsg)).toBe(7)
    expect(apidOf(toLabMsg)).toBe(0)
    expect(apidOf(parseErr)).toBe(9)
    expect(apidOf(evsLong)).toBe(9)
  })

  it('matchesSearch scans JSON', () => {
    expect(matchesSearch(esMsg, 'heap')).toBe(true)
    expect(matchesSearch(esMsg, 'nomatch')).toBe(false)
    expect(matchesSearch(esMsg, '')).toBe(true)
  })

  it('filterEntries by kind', () => {
    const entries: TlmEntry[] = [
      { seq: 1, message: esMsg },
      { seq: 2, message: toLabMsg },
      { seq: 3, message: parseErr },
      { seq: 4, message: evsLong },
    ]
    expect(filterEntries(entries, 'es_hk_v1', '', '').length).toBe(1)
    expect(filterEntries(entries, 'to_lab_hk_v1', '', '').length).toBe(1)
    expect(filterEntries(entries, 'parse_error', '', '').length).toBe(1)
    expect(filterEntries(entries, 'evs_long_event_v1', '', '').length).toBe(1)
    expect(filterEntries(entries, 'all', '', '').length).toBe(4)
  })

  it('filterEntries hides parse_error when requested', () => {
    const entries: TlmEntry[] = [
      { seq: 1, message: esMsg },
      { seq: 2, message: parseErr },
      { seq: 3, message: toLabMsg },
      { seq: 4, message: evsLong },
    ]
    expect(filterEntries(entries, 'all', '', '', true).length).toBe(3)
    expect(filterEntries(entries, 'parse_error', '', '', true).length).toBe(0)
  })

  it('filterEntries by apid', () => {
    const entries: TlmEntry[] = [
      { seq: 1, message: esMsg },
      { seq: 2, message: parseErr },
    ]
    expect(filterEntries(entries, 'all', '7', '').length).toBe(1)
    expect(filterEntries(entries, 'all', '9', '').length).toBe(1)
  })

  it('pageSlice and totalPageCount', () => {
    const items = [1, 2, 3, 4, 5]
    expect(totalPageCount(5, 2)).toBe(3)
    expect(pageSlice(items, 2, 0)).toEqual([1, 2])
    expect(pageSlice(items, 2, 2)).toEqual([5])
    expect(pageSlice([], 10, 0)).toEqual([])
  })

  it('summaryLine covers kinds', () => {
    expect(summaryLine(esMsg)).toContain('ES HK')
    expect(summaryLine(toLabMsg)).toContain('TO_LAB HK')
    expect(summaryLine(parseErr)).toBe('bad')
    expect(summaryLine(evsLong)).toContain('CFE_EVS')
  })
})
