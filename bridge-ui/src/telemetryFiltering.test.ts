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

const parseErr = {
  kind: 'parse_error' as const,
  received_at: '2026-01-02T00:00:00Z',
  raw_len: 10,
  primary: { apid: 9, packet_type: 0, sequence_count: 0 },
  message: 'bad',
  hex_preview: 'aa bb',
}

describe('telemetryFiltering', () => {
  it('apidOf reads primary', () => {
    expect(apidOf(esMsg)).toBe(7)
    expect(apidOf(parseErr)).toBe(9)
  })

  it('matchesSearch scans JSON', () => {
    expect(matchesSearch(esMsg, 'heap')).toBe(true)
    expect(matchesSearch(esMsg, 'nomatch')).toBe(false)
    expect(matchesSearch(esMsg, '')).toBe(true)
  })

  it('filterEntries by kind', () => {
    const entries: TlmEntry[] = [
      { seq: 1, message: esMsg },
      { seq: 2, message: parseErr },
    ]
    expect(filterEntries(entries, 'es_hk_v1', '', '').length).toBe(1)
    expect(filterEntries(entries, 'parse_error', '', '').length).toBe(1)
    expect(filterEntries(entries, 'all', '', '').length).toBe(2)
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
    expect(summaryLine(parseErr)).toBe('bad')
  })
})
