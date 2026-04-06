/* eslint-disable react-refresh/only-export-components */
import { observer } from 'mobx-react-lite'

import type { AlertStore } from '../../stores/alertStore'
import type { TelemetryStore } from '../../stores/telemetryStore'
import { AlertFeedPanel } from './AlertFeedPanel'
import { Panel } from '../ui/Panel'
import { StatusBadge } from '../ui/StatusBadge'

function Row({ label, value }: { label: string; value: React.ReactNode }) {
  return (
    <div style={{ display: 'flex', justifyContent: 'space-between', padding: '0.2rem 0', gap: '1rem', fontSize: '0.85rem' }}>
      <span style={{ opacity: 0.7 }}>{label}</span>
      <span style={{ fontVariantNumeric: 'tabular-nums' }}>{value}</span>
    </div>
  )
}

function pct(used: number, total: number): string {
  if (!total) return '—'
  return `${((used / total) * 100).toFixed(1)}%`
}

function formatBytes(v: number): string {
  if (v >= 1_048_576) return `${(v / 1_048_576).toFixed(1)} MB`
  if (v >= 1024) return `${(v / 1024).toFixed(0)} KB`
  return `${v} B`
}

export const HealthDashboard = observer(function HealthDashboard({
  telemetry,
  alerts,
}: {
  telemetry: TelemetryStore
  alerts: AlertStore
}) {
  const hk = telemetry.lastEsHk?.es_hk ?? null
  const tolab = telemetry.lastToLabHk?.to_lab_hk ?? null
  const dash = '—'

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: '1rem' }}>
      <AlertFeedPanel store={alerts} />
      <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fill, minmax(280px, 1fr))', gap: '1rem' }}>

      <Panel title="Flight Computer Status">
        <Row label="Processor resets" value={hk ? hk.processor_resets : dash} />
        <Row label="Max allowed resets" value={hk ? hk.max_processor_resets : dash} />
        <Row label="Reset type" value={hk ? hk.reset_type : dash} />
        <Row label="Reset subtype" value={hk ? hk.reset_subtype : dash} />
        <Row label="Boot source" value={hk ? hk.boot_source : dash} />
      </Panel>

      <Panel title="Memory Health">
        <Row label="Heap free" value={hk ? formatBytes(hk.heap_bytes_free) : dash} />
        <Row label="Max block" value={hk ? formatBytes(hk.heap_max_block_size) : dash} />
        <Row
          label="Fragmentation"
          value={
            hk && hk.heap_bytes_free > 0
              ? `${(100 - (hk.heap_max_block_size / hk.heap_bytes_free) * 100).toFixed(1)}%`
              : dash
          }
        />
        <Row label="Blocks free" value={hk ? hk.heap_blocks_free : dash} />
      </Panel>

      <Panel title="Syslog Pressure">
        <Row label="Bytes used" value={hk ? formatBytes(hk.syslog_bytes_used) : dash} />
        <Row label="Total size" value={hk ? formatBytes(hk.syslog_size) : dash} />
        <Row label="Fill %" value={hk ? pct(hk.syslog_bytes_used, hk.syslog_size) : dash} />
        <Row label="Entries" value={hk ? hk.syslog_entries : dash} />
      </Panel>

      <Panel title="App Registry">
        <Row label="Core apps" value={hk ? hk.registered_core_apps : dash} />
        <Row label="External apps" value={hk ? hk.registered_external_apps : dash} />
        <Row label="Tasks" value={hk ? hk.registered_tasks : dash} />
        <Row label="Libs" value={hk ? hk.registered_libs : dash} />
      </Panel>

      <Panel title="Command Health">
        <Row label="ES cmd counter" value={hk ? hk.command_counter : dash} />
        <Row
          label="ES cmd errors"
          value={
            hk ? (
              <span style={{ color: hk.command_error_counter > 0 ? 'var(--color-error, #f44336)' : undefined }}>
                {hk.command_error_counter}
              </span>
            ) : dash
          }
        />
        <Row label="TO_LAB cmd counter" value={tolab ? tolab.command_counter : dash} />
        <Row
          label="TO_LAB cmd errors"
          value={
            tolab ? (
              <span style={{ color: tolab.command_error_counter > 0 ? 'var(--color-error, #f44336)' : undefined }}>
                {tolab.command_error_counter}
              </span>
            ) : dash
          }
        />
      </Panel>

      <Panel title="Bridge Health">
        <Row
          label="WebSocket"
          value={
            <StatusBadge ok={telemetry.bridgeLinkLive}>
              {telemetry.bridgeLinkLive ? 'Live' : 'Offline'}
            </StatusBadge>
          }
        />
        <Row
          label="Downlink"
          value={
            <StatusBadge ok={telemetry.downlinkLive}>
              {telemetry.downlinkLive ? 'Live' : 'Offline'}
            </StatusBadge>
          }
        />
        <Row label="Packets (session)" value={telemetry.packetCount} />
        <Row label="Active alerts" value={alerts.alerts.length} />
      </Panel>

      </div>
    </div>
  )
})
