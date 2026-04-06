import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { TelemetryStore } from '../../stores/telemetryStore'

import { TelemetryOverview } from './TelemetryOverview'

import * as api from '../../api'

describe('TelemetryOverview', () => {
  it('shows offline when not connected', () => {
    const store = new TelemetryStore()
    store.connected = false
    render(<TelemetryOverview store={store} />)
    expect(screen.getAllByText('Offline')).toHaveLength(2)
  })

  it('renders ES HK fields when message present', () => {
    const store = new TelemetryStore()
    store.connected = true
    store.appendMessage({
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
    })
    render(<TelemetryOverview store={store} />)
    expect(screen.getByText('7.8.9.0')).toBeInTheDocument()
    expect(screen.getByText(/10 \/ 2/)).toBeInTheDocument()
  })

  it('renders TO_LAB HK when message present', () => {
    const store = new TelemetryStore()
    store.connected = true
    store.appendMessage({
      kind: 'to_lab_hk_v1',
      received_at: '2026-04-06T12:00:00.000Z',
      raw_len: 22,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      to_lab_hk: { command_counter: 9, command_error_counter: 2 },
    })
    render(<TelemetryOverview store={store} />)
    expect(screen.getByRole('heading', { name: 'TO_LAB HK' })).toBeInTheDocument()
    expect(screen.getByText('9')).toBeInTheDocument()
    expect(screen.getByText('2')).toBeInTheDocument()
  })

  it('toggles TO_LAB output via API', async () => {
    const store = new TelemetryStore()
    store.connected = true
    const spy = vi
      .spyOn(api, 'setToLabOutputEnabled')
      .mockResolvedValue({ bytes_sent: 8, wire_length: 8 })

    render(<TelemetryOverview store={store} />)
    const btn = screen.getByRole('button', { name: 'Enable TO_LAB output' })
    fireEvent.click(btn)
    await waitFor(() => expect(spy).toHaveBeenCalledWith(true))
  })

  it('reverts button state on disable even when lastToLabHk stays cached', async () => {
    const store = new TelemetryStore()
    store.connected = true
    store.appendMessage({
      kind: 'to_lab_hk_v1',
      received_at: '2026-04-06T12:00:00.000Z',
      raw_len: 22,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      to_lab_hk: { command_counter: 9, command_error_counter: 2 },
    })

    const spy = vi
      .spyOn(api, 'setToLabOutputEnabled')
      .mockResolvedValue({ bytes_sent: 8, wire_length: 8 })

    render(<TelemetryOverview store={store} />)

    // Starts as enabled because we've seen TO_LAB HK once.
    const disableBtn = screen.getByRole('button', { name: 'Disable TO_LAB output' })
    expect(disableBtn).toHaveTextContent('On')

    fireEvent.click(disableBtn)
    await waitFor(() => expect(spy).toHaveBeenCalledWith(false))
    // Spy fires before React applies setToLabDesiredEnabled; wait for idle toggle UI.
    const enableBtn = await screen.findByRole('button', { name: 'Enable TO_LAB output' })
    await waitFor(() => expect(enableBtn).toHaveTextContent('Off'))

    fireEvent.click(enableBtn)
    await waitFor(() => expect(spy).toHaveBeenCalledWith(true))
  })

  it('keeps prior toggle state and shows error banner when TO_LAB toggle fails', async () => {
    const store = new TelemetryStore()
    store.connected = true
    store.appendMessage({
      kind: 'to_lab_hk_v1',
      received_at: '2026-04-06T12:00:00.000Z',
      raw_len: 22,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      to_lab_hk: { command_counter: 9, command_error_counter: 2 },
    })

    vi.spyOn(api, 'setToLabOutputEnabled').mockRejectedValue(new Error('boom'))

    render(<TelemetryOverview store={store} />)

    const disableBtn = screen.getByRole('button', { name: 'Disable TO_LAB output' })
    fireEvent.click(disableBtn)

    expect(await screen.findByRole('alert')).toHaveTextContent('boom')
    // State should remain "On" because request failed.
    expect(screen.getByRole('button', { name: 'Disable TO_LAB output' })).toHaveTextContent('On')
  })
})
