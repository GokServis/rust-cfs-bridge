import { observer } from 'mobx-react-lite'
import { useMemo, useState } from 'react'

import type { TelemetryStore } from '../../stores/telemetryStore'
import type { TlmMessage } from '../../telemetryTypes'
import { setToLabOutputEnabled } from '../../api'
import { Card } from '../ui/Card'
import { Panel } from '../ui/Panel'
import { StatusBadge } from '../ui/StatusBadge'

import { EsHkPanel } from './EsHkPanel'
import { ParseErrorPanel } from './ParseErrorPanel'
import { ToLabHkPanel } from './ToLabHkPanel'
import { StaleWarning } from './StaleWarning'

import './TelemetryOverview.css'

function serverReceivedAt(msg: TlmMessage | null): string | null {
  if (!msg) return null
  return msg.received_at
}

export const TelemetryOverview = observer(function TelemetryOverview({
  store,
}: {
  store: TelemetryStore
}) {
  const {
    bridgeLinkLive,
    downlinkLive,
    lastReceivedAt,
    lastMessage,
    lastEsHk,
    lastToLabHk,
    error,
    packetCount,
  } = store
  const serverTs = serverReceivedAt(lastMessage)
  const [toLabPending, setToLabPending] = useState(false)
  const [toLabError, setToLabError] = useState<string | null>(null)

  const inferredToLabEnabled = Boolean(lastToLabHk)
  const [toLabDesiredEnabled, setToLabDesiredEnabled] = useState<boolean | null>(null)
  const toLabEnabled = useMemo(
    () => toLabDesiredEnabled ?? inferredToLabEnabled,
    [toLabDesiredEnabled, inferredToLabEnabled],
  )

  return (
    <div className="telemetry-overview">
      <div className="telemetry-hero">
        <p className="telemetry-hero__label">Downlink</p>
        <p className="telemetry-hero__lede" id="tlm-desc">
          Executive Services HK (UDP → WebSocket). Heap and app counts reflect cFE ES housekeeping when
          packets are valid.
        </p>
      </div>

      <Card>
        <h2 className="telemetry-hero__label" id="tlm-heading">
          Live telemetry
        </h2>
        <div className="telemetry-metrics">
          <div className="telemetry-metrics__row">
            <span className="telemetry-metrics__k">Bridge (API)</span>
            <StatusBadge ok={bridgeLinkLive}>{bridgeLinkLive ? 'Live' : 'Offline'}</StatusBadge>
          </div>
          <div className="telemetry-metrics__row">
            <span className="telemetry-metrics__k">Downlink</span>
            <StatusBadge ok={downlinkLive}>{downlinkLive ? 'Live' : 'Offline'}</StatusBadge>
          </div>
          <div className="telemetry-metrics__row">
            <span className="telemetry-metrics__k">Packets (session)</span>
            <span className="telemetry-metrics__v">{packetCount}</span>
          </div>
          <div className="telemetry-metrics__row">
            <span className="telemetry-metrics__k">Last (browser)</span>
            <span className="telemetry-metrics__v">{lastReceivedAt ?? '—'}</span>
          </div>
          <div className="telemetry-metrics__row">
            <span className="telemetry-metrics__k">Last (server)</span>
            <span className="telemetry-metrics__v">{serverTs ?? '—'}</span>
          </div>
        </div>
        <div className="telemetry-metrics telemetry-metrics--controls">
          <div className="telemetry-metrics__row">
            <span className="telemetry-metrics__k">TO_LAB output</span>
            <button
              type="button"
              className={`telemetry-toggle ${toLabEnabled ? 'telemetry-toggle--on' : 'telemetry-toggle--off'}`}
              disabled={toLabPending}
              onClick={async () => {
                setToLabPending(true)
                setToLabError(null)
                const nextEnabled = !toLabEnabled
                try {
                  await setToLabOutputEnabled(nextEnabled)
                  setToLabDesiredEnabled(nextEnabled)
                } catch (e: unknown) {
                  setToLabError(e instanceof Error ? e.message : 'TO_LAB toggle failed')
                } finally {
                  setToLabPending(false)
                }
              }}
              aria-label={toLabEnabled ? 'Disable TO_LAB output' : 'Enable TO_LAB output'}
            >
              {toLabPending ? 'Working…' : toLabEnabled ? 'On' : 'Off'}
            </button>
          </div>
        </div>
        {toLabError ? (
          <div className="banner-telemetry-error" role="alert">
            {toLabError}
          </div>
        ) : null}
        {error ? (
          <div className="banner-telemetry-error" role="alert">
            {error}
          </div>
        ) : null}
        <StaleWarning serverIso={serverTs} />
      </Card>

      <Panel title="ES HK" className="telemetry-grid">
        {lastEsHk ? (
          <EsHkPanel msg={lastEsHk} />
        ) : (
          <dl className="telemetry-dl">
            <div className="telemetry-dl__row">
              <dt>Status</dt>
              <dd>—</dd>
            </div>
          </dl>
        )}
      </Panel>

      <Panel title="TO_LAB HK" className="telemetry-grid">
        {lastToLabHk ? (
          <ToLabHkPanel msg={lastToLabHk} />
        ) : (
          <dl className="telemetry-dl">
            <div className="telemetry-dl__row">
              <dt>Command counter</dt>
              <dd>—</dd>
            </div>
            <div className="telemetry-dl__row">
              <dt>Command error counter</dt>
              <dd>—</dd>
            </div>
          </dl>
        )}
      </Panel>

      {lastMessage?.kind === 'parse_error' ? <ParseErrorPanel msg={lastMessage} /> : null}
    </div>
  )
})
