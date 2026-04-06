import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { TelemetryStore } from '../../stores/telemetryStore'

import { TelemetryOverview } from './TelemetryOverview'

describe('TelemetryOverview', () => {
  it('shows offline when not connected', () => {
    const store = new TelemetryStore()
    store.connected = false
    render(<TelemetryOverview store={store} />)
    expect(screen.getByText('Offline')).toBeInTheDocument()
  })

  it('renders ES HK fields when message present', () => {
    const store = new TelemetryStore()
    store.connected = true
    store.packetCount = 1
    store.lastMessage = {
      kind: 'es_hk_v1',
      received_at: '2026-04-06T12:00:00.000Z',
      raw_len: 180,
      primary: { apid: 5, packet_type: 0, sequence_count: 2 },
      es_hk: {
        command_counter: 3,
        command_error_counter: 0,
        cfe_core_checksum: 0,
        cfe_version: [7, 8, 9, 0],
        osal_version: [1, 0, 0, 0],
        psp_version: [2, 0, 0, 0],
        syslog_bytes_used: 0,
        syslog_size: 0,
        syslog_entries: 0,
        syslog_mode: 0,
        registered_core_apps: 10,
        registered_external_apps: 2,
        registered_tasks: 20,
        registered_libs: 3,
        reset_type: 0,
        reset_subtype: 0,
        processor_resets: 0,
        max_processor_resets: 0,
        boot_source: 0,
        perf_state: 0,
        perf_mode: 0,
        perf_trigger_count: 0,
        heap_bytes_free: 1000,
        heap_blocks_free: 5,
        heap_max_block_size: 500,
      },
    }
    render(<TelemetryOverview store={store} />)
    expect(screen.getByText('7.8.9.0')).toBeInTheDocument()
    expect(screen.getByText(/10 \/ 2/)).toBeInTheDocument()
  })

  it('renders TO_LAB HK when message present', () => {
    const store = new TelemetryStore()
    store.connected = true
    store.packetCount = 1
    store.lastMessage = {
      kind: 'to_lab_hk_v1',
      received_at: '2026-04-06T12:00:00.000Z',
      raw_len: 22,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      to_lab_hk: { command_counter: 9, command_error_counter: 2 },
    }
    render(<TelemetryOverview store={store} />)
    expect(screen.getByRole('heading', { name: 'TO_LAB HK' })).toBeInTheDocument()
    expect(screen.getByText('9')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
  })
})
