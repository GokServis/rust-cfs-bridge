import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { AlertStore } from '../../stores/alertStore'
import { AlertFeedPanel } from './AlertFeedPanel'

const lowHeapEsHk = {
  kind: 'es_hk_v1' as const,
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
    heap_bytes_free: 50_000,
    heap_blocks_free: 10,
    heap_max_block_size: 20_000,
  },
}

function storeWithWarnAlert(): AlertStore {
  const store = new AlertStore()
  store.evaluate(lowHeapEsHk)
  return store
}

describe('AlertFeedPanel', () => {
  it('shows no alerts yet when empty', () => {
    const store = new AlertStore()
    render(<AlertFeedPanel store={store} />)
    expect(screen.getByText(/no alerts yet/i)).toBeInTheDocument()
  })

  it('renders severity label for a warn alert', () => {
    const store = storeWithWarnAlert()
    render(<AlertFeedPanel store={store} />)
    expect(screen.getByText('WARN')).toBeInTheDocument()
    expect(screen.getByText(/heap/i)).toBeInTheDocument()
  })

  it('dismiss removes the alert', () => {
    const store = storeWithWarnAlert()
    render(<AlertFeedPanel store={store} />)
    expect(store.alerts).toHaveLength(1)
    const btn = screen.getByRole('button', { name: /dismiss/i })
    fireEvent.click(btn)
    expect(store.alerts).toHaveLength(0)
    expect(screen.getByText(/no alerts yet/i)).toBeInTheDocument()
  })

  it('clear all empties alerts', () => {
    const store = storeWithWarnAlert()
    render(<AlertFeedPanel store={store} />)
    fireEvent.click(screen.getByRole('button', { name: /clear all/i }))
    expect(store.alerts).toHaveLength(0)
  })

  it('filters by severity', () => {
    const store = new AlertStore()
    store.evaluate(lowHeapEsHk)
    // Heap above threshold so this evaluation only adds the critical (no second heap warn).
    store.evaluate({
      ...lowHeapEsHk,
      es_hk: { ...lowHeapEsHk.es_hk, heap_bytes_free: 500_000, processor_resets: 1 },
    })

    render(<AlertFeedPanel store={store} />)
    expect(screen.getByText('CRITICAL')).toBeInTheDocument()
    expect(screen.getByText('WARN')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText(/filter alerts by severity/i), { target: { value: 'critical' } })
    expect(screen.getByText('CRITICAL')).toBeInTheDocument()
    expect(screen.queryByText('WARN')).not.toBeInTheDocument()
  })

  it('paginates with next and previous', () => {
    const store = new AlertStore()
    store.setAlertPageSize(2)
    for (let i = 0; i < 5; i++) {
      store.evaluate({
        ...lowHeapEsHk,
        es_hk: { ...lowHeapEsHk.es_hk, heap_bytes_free: 50_000 - i },
      })
    }
    expect(store.alerts).toHaveLength(5)

    render(<AlertFeedPanel store={store} />)
    expect(screen.getByText(/page 1 of 3/i)).toBeInTheDocument()

    const next = screen.getByRole('button', { name: /^next$/i })
    const prev = screen.getByRole('button', { name: /^previous$/i })
    expect(prev).toBeDisabled()
    fireEvent.click(next)
    expect(screen.getByText(/page 2 of 3/i)).toBeInTheDocument()
    expect(prev).not.toBeDisabled()
    fireEvent.click(prev)
    expect(screen.getByText(/page 1 of 3/i)).toBeInTheDocument()
  })
})
