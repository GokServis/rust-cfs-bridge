import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'

import { AlertStore } from '../../stores/alertStore'
import { TelemetryStore } from '../../stores/telemetryStore'
import { HealthDashboard } from './HealthDashboard'

const esHkMsg = {
  kind: 'es_hk_v1' as const,
  received_at: '2026-01-01T00:00:00.000Z',
  raw_len: 180,
  primary: { apid: 0, packet_type: 0, sequence_count: 0 },
  es_hk: {
    command_counter: 3,
    command_error_counter: 0,
    cfe_core_checksum: 0,
    cfe_version: [7, 0, 0, 0],
    osal_version: [0, 0, 0, 0],
    psp_version: [0, 0, 0, 0],
    syslog_bytes_used: 512,
    syslog_size: 1024,
    syslog_entries: 10,
    syslog_mode: 0,
    registered_core_apps: 6,
    registered_external_apps: 2,
    registered_tasks: 14,
    registered_libs: 3,
    reset_type: 1,
    reset_subtype: 0,
    processor_resets: 0,
    max_processor_resets: 3,
    boot_source: 0,
    perf_state: 0,
    perf_mode: 0,
    perf_trigger_count: 0,
    heap_bytes_free: 600_000,
    heap_blocks_free: 50,
    heap_max_block_size: 300_000,
  },
}

describe('HealthDashboard', () => {
  it('renders all panel headings', () => {
    const telemetry = new TelemetryStore()
    const alerts = new AlertStore()
    render(<HealthDashboard telemetry={telemetry} alerts={alerts} />)
    expect(screen.getByRole('heading', { name: /^alerts$/i })).toBeInTheDocument()
    expect(screen.getByLabelText(/filter alerts by severity/i)).toBeInTheDocument()
    expect(screen.getByText(/flight computer/i)).toBeInTheDocument()
    expect(screen.getByText(/memory/i)).toBeInTheDocument()
    expect(screen.getByText(/syslog/i)).toBeInTheDocument()
    expect(screen.getByText(/command health/i)).toBeInTheDocument()
    expect(screen.getByText(/app registry/i)).toBeInTheDocument()
    expect(screen.getByText(/bridge/i)).toBeInTheDocument()
  })

  it('shows — placeholders when no telemetry received', () => {
    const telemetry = new TelemetryStore()
    const alerts = new AlertStore()
    render(<HealthDashboard telemetry={telemetry} alerts={alerts} />)
    // Multiple dash placeholders expected
    expect(screen.getAllByText('—').length).toBeGreaterThan(0)
  })

  it('shows ES HK values when telemetry is present', () => {
    const telemetry = new TelemetryStore()
    const alerts = new AlertStore()
    telemetry.appendMessage(esHkMsg)
    render(<HealthDashboard telemetry={telemetry} alerts={alerts} />)
    // registered_core_apps: 6 — unique numeric value in the rendered output
    expect(screen.getByText('6')).toBeInTheDocument()
    // registered_external_apps: 2
    expect(screen.getByText('2')).toBeInTheDocument()
  })

  it('shows bridge offline when not connected', () => {
    const telemetry = new TelemetryStore()
    const alerts = new AlertStore()
    render(<HealthDashboard telemetry={telemetry} alerts={alerts} />)
    // Both WebSocket and Downlink show Offline when store is not connected
    expect(screen.getAllByText('Offline').length).toBeGreaterThanOrEqual(1)
  })
})
