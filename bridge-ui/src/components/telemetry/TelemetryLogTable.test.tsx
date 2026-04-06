import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it } from 'vitest'

import { TelemetryStore } from '../../stores/telemetryStore'

import { TelemetryLogTable } from './TelemetryLogTable'

describe('TelemetryLogTable', () => {
  it('renders empty state when no matches', () => {
    const store = new TelemetryStore()
    store.appendMessage({
      kind: 'es_hk_v1',
      received_at: '2026-01-01T00:00:00Z',
      raw_len: 180,
      primary: { apid: 1, packet_type: 0, sequence_count: 0 },
      es_hk: {
        command_counter: 0,
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
        heap_bytes_free: 0,
        heap_blocks_free: 0,
        heap_max_block_size: 0,
      },
    })
    store.setKindFilter('parse_error')
    render(<TelemetryLogTable store={store} />)
    expect(screen.getByText(/No rows match/i)).toBeInTheDocument()
  })

  it('shows row when kind matches', () => {
    const store = new TelemetryStore()
    store.appendMessage({
      kind: 'parse_error',
      received_at: '2026-01-01T00:00:00Z',
      raw_len: 4,
      primary: { apid: 42, packet_type: 0, sequence_count: 0 },
      message: 'oops',
      hex_preview: '00',
    })
    render(<TelemetryLogTable store={store} />)
    expect(screen.getByText('oops')).toBeInTheDocument()
    expect(screen.getByText('42')).toBeInTheDocument()
  })

  it('pager moves between pages', async () => {
    const user = userEvent.setup()
    const store = new TelemetryStore()
    store.setPageSize(1)
    for (let i = 0; i < 3; i++) {
      store.appendMessage({
        kind: 'parse_error',
        received_at: `2026-01-0${i + 1}T00:00:00Z`,
        raw_len: 1,
        primary: null,
        message: `m${i}`,
        hex_preview: '00',
      })
    }
    render(<TelemetryLogTable store={store} />)
    expect(screen.getByText('m0')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: /Next/i }))
    expect(screen.getByText('m1')).toBeInTheDocument()
    await user.click(screen.getByRole('button', { name: /Previous/i }))
    expect(screen.getByText('m0')).toBeInTheDocument()
  })
})
