import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { AlertStore } from '../../stores/alertStore'
import { AlertBanner } from './AlertBanner'

function makeStoreWithAlert(severity: 'warn' | 'error' | 'critical', message: string) {
  const store = new AlertStore()
  // Trigger an alert by evaluating a message that produces the desired severity
  if (severity === 'error') {
    store.evaluate({
      kind: 'es_hk_v1',
      received_at: '2026-01-01T00:00:00.000Z',
      raw_len: 180,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      es_hk: {
        command_counter: 0,
        command_error_counter: 1,
        cfe_core_checksum: 0,
        cfe_version: [1, 0, 0, 0],
        osal_version: [0, 0, 0, 0],
        psp_version: [0, 0, 0, 0],
        syslog_bytes_used: 0,
        syslog_size: 1024,
        syslog_entries: 0,
        syslog_mode: 0,
        registered_core_apps: 6,
        registered_external_apps: 2,
        registered_tasks: 8,
        registered_libs: 2,
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
    })
  } else if (severity === 'critical') {
    store.evaluate({
      kind: 'es_hk_v1',
      received_at: '2026-01-01T00:00:00.000Z',
      raw_len: 180,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      es_hk: {
        command_counter: 0,
        command_error_counter: 0,
        cfe_core_checksum: 0,
        cfe_version: [1, 0, 0, 0],
        osal_version: [0, 0, 0, 0],
        psp_version: [0, 0, 0, 0],
        syslog_bytes_used: 0,
        syslog_size: 1024,
        syslog_entries: 0,
        syslog_mode: 0,
        registered_core_apps: 6,
        registered_external_apps: 2,
        registered_tasks: 8,
        registered_libs: 2,
        reset_type: 0,
        reset_subtype: 0,
        processor_resets: 1,
        max_processor_resets: 5,
        boot_source: 0,
        perf_state: 0,
        perf_mode: 0,
        perf_trigger_count: 0,
        heap_bytes_free: 500_000,
        heap_blocks_free: 100,
        heap_max_block_size: 200_000,
      },
    })
  } else {
    store.evaluate({
      kind: 'es_hk_v1',
      received_at: '2026-01-01T00:00:00.000Z',
      raw_len: 180,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      es_hk: {
        command_counter: 0,
        command_error_counter: 0,
        cfe_core_checksum: 0,
        cfe_version: [1, 0, 0, 0],
        osal_version: [0, 0, 0, 0],
        psp_version: [0, 0, 0, 0],
        syslog_bytes_used: 0,
        syslog_size: 1024,
        syslog_entries: 0,
        syslog_mode: 0,
        registered_core_apps: 6,
        registered_external_apps: 2,
        registered_tasks: 8,
        registered_libs: 2,
        reset_type: 0,
        reset_subtype: 0,
        processor_resets: 0,
        max_processor_resets: 5,
        boot_source: 0,
        perf_state: 0,
        perf_mode: 0,
        perf_trigger_count: 0,
        heap_bytes_free: 50_000, // below threshold → warn
        heap_blocks_free: 10,
        heap_max_block_size: 20_000,
      },
    })
  }
  void message
  return store
}

describe('AlertBanner', () => {
  it('renders nothing when there are no alerts', () => {
    const store = new AlertStore()
    const { container } = render(<AlertBanner store={store} />)
    expect(container.firstChild).toBeNull()
  })

  it('renders an error alert with ERROR label', () => {
    const store = makeStoreWithAlert('error', '')
    render(<AlertBanner store={store} />)
    expect(screen.getByRole('alert')).toBeInTheDocument()
    expect(screen.getByText('ERROR')).toBeInTheDocument()
  })

  it('renders a critical alert with CRITICAL label', () => {
    const store = makeStoreWithAlert('critical', '')
    render(<AlertBanner store={store} />)
    expect(screen.getByText('CRITICAL')).toBeInTheDocument()
  })

  it('renders a warn alert with WARN label', () => {
    const store = makeStoreWithAlert('warn', '')
    render(<AlertBanner store={store} />)
    expect(screen.getByText('WARN')).toBeInTheDocument()
  })

  it('dismiss button calls dismissAlert on the store', () => {
    const store = makeStoreWithAlert('error', '')
    render(<AlertBanner store={store} />)
    expect(store.alerts).toHaveLength(1)
    const btn = screen.getByRole('button', { name: /dismiss/i })
    fireEvent.click(btn)
    expect(store.alerts).toHaveLength(0)
  })
})
