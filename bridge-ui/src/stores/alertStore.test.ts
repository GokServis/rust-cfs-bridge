import { describe, expect, it } from 'vitest'

import { AlertStore } from './alertStore'

const baseEsHk = {
  kind: 'es_hk_v1' as const,
  received_at: '2026-01-01T00:00:00.000Z',
  raw_len: 180,
  primary: { apid: 0, packet_type: 0, sequence_count: 0 },
  es_hk: {
    command_counter: 0,
    command_error_counter: 0,
    cfe_core_checksum: 0,
    cfe_version: [1, 2, 3, 4],
    osal_version: [0, 0, 0, 0],
    psp_version: [0, 0, 0, 0],
    syslog_bytes_used: 0,
    syslog_size: 1024,
    syslog_entries: 0,
    syslog_mode: 0,
    registered_core_apps: 0,
    registered_external_apps: 0,
    registered_tasks: 0,
    registered_libs: 0,
    reset_type: 0,
    reset_subtype: 0,
    processor_resets: 0,
    max_processor_resets: 5,
    boot_source: 0,
    perf_state: 0,
    perf_mode: 0,
    perf_trigger_count: 0,
    heap_bytes_free: 500_000,
    heap_blocks_free: 100,
    heap_max_block_size: 200_000,
  },
}

const baseToLab = {
  kind: 'to_lab_hk_v1' as const,
  received_at: '2026-01-01T00:00:00.000Z',
  raw_len: 20,
  primary: { apid: 0x80, packet_type: 0, sequence_count: 0 },
  to_lab_hk: { command_counter: 0, command_error_counter: 0 },
}

describe('AlertStore', () => {
  it('starts with no alerts', () => {
    const store = new AlertStore()
    expect(store.alerts).toHaveLength(0)
  })

  it('fires ERROR alert when ES command_error_counter increments', () => {
    const store = new AlertStore()
    store.evaluate(baseEsHk)
    store.evaluate({ ...baseEsHk, es_hk: { ...baseEsHk.es_hk, command_error_counter: 1 } })
    const errors = store.alerts.filter(a => a.severity === 'error')
    expect(errors).toHaveLength(1)
    expect(errors[0].message).toMatch(/command error/i)
  })

  it('fires CRITICAL alert when processor_resets increments', () => {
    const store = new AlertStore()
    store.evaluate(baseEsHk)
    store.evaluate({ ...baseEsHk, es_hk: { ...baseEsHk.es_hk, processor_resets: 1 } })
    const crits = store.alerts.filter(a => a.severity === 'critical')
    expect(crits).toHaveLength(1)
    expect(crits[0].message).toMatch(/processor reset/i)
  })

  it('fires WARN alert when heap_bytes_free drops below threshold', () => {
    const store = new AlertStore()
    // below 128 KB threshold
    store.evaluate({ ...baseEsHk, es_hk: { ...baseEsHk.es_hk, heap_bytes_free: 50_000 } })
    const warns = store.alerts.filter(a => a.severity === 'warn')
    expect(warns).toHaveLength(1)
    expect(warns[0].message).toMatch(/heap/i)
  })

  it('does not fire heap alert when heap is above threshold', () => {
    const store = new AlertStore()
    store.evaluate(baseEsHk) // heap_bytes_free = 500_000
    expect(store.alerts.filter(a => a.severity === 'warn')).toHaveLength(0)
  })

  it('fires WARN alert when TO_LAB command_error_counter increments', () => {
    const store = new AlertStore()
    store.evaluate(baseToLab)
    store.evaluate({ ...baseToLab, to_lab_hk: { command_counter: 1, command_error_counter: 1 } })
    const warns = store.alerts.filter(a => a.message.toLowerCase().includes('to_lab'))
    expect(warns).toHaveLength(1)
  })

  it('does not fire duplicate error alert on repeated same counter value', () => {
    const store = new AlertStore()
    const msgWithError = { ...baseEsHk, es_hk: { ...baseEsHk.es_hk, command_error_counter: 1 } }
    store.evaluate(msgWithError)
    store.evaluate(msgWithError)
    expect(store.alerts.filter(a => a.severity === 'error')).toHaveLength(1)
  })

  it('dismissAlert removes the alert by id', () => {
    const store = new AlertStore()
    store.evaluate({ ...baseEsHk, es_hk: { ...baseEsHk.es_hk, processor_resets: 2 } })
    expect(store.alerts).toHaveLength(1)
    store.dismissAlert(store.alerts[0].id)
    expect(store.alerts).toHaveLength(0)
  })

  it('clearAll empties the alerts list', () => {
    const store = new AlertStore()
    store.evaluate({ ...baseEsHk, es_hk: { ...baseEsHk.es_hk, processor_resets: 1 } })
    store.clearAll()
    expect(store.alerts).toHaveLength(0)
  })

  it('fires ERROR alert when command_ack result is rejected', () => {
    const store = new AlertStore()
    store.evaluate({
      kind: 'command_ack',
      received_at: '2026-01-01T00:00:00.000Z',
      name: 'CMD_PING',
      sequence_count: 42,
      result: 'rejected',
      latency_ms: 150,
    })
    const errors = store.alerts.filter(a => a.severity === 'error')
    expect(errors).toHaveLength(1)
    expect(errors[0].message).toMatch(/CMD_PING/)
    expect(errors[0].message).toMatch(/rejected/i)
  })

  it('does not fire alert when command_ack result is accepted', () => {
    const store = new AlertStore()
    store.evaluate({
      kind: 'command_ack',
      received_at: '2026-01-01T00:00:00.000Z',
      name: 'CMD_PING',
      sequence_count: 42,
      result: 'accepted',
      latency_ms: 80,
    })
    expect(store.alerts).toHaveLength(0)
  })
})
