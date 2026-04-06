import { observer } from 'mobx-react-lite'

import type { TelemetryStore } from '../../stores/telemetryStore'
import type { TlmMessage } from '../../telemetryTypes'
import { Card } from '../ui/Card'
import { Panel } from '../ui/Panel'
import { StatusBadge } from '../ui/StatusBadge'

import { EsHkPanel } from './EsHkPanel'
import { ParseErrorPanel } from './ParseErrorPanel'
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
  const { connected, lastReceivedAt, lastMessage, error, packetCount } = store
  const msg = lastMessage
  const serverTs = serverReceivedAt(msg)

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
            <span className="telemetry-metrics__k">Link</span>
            <StatusBadge ok={connected}>{connected ? 'Live' : 'Offline'}</StatusBadge>
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
        {error ? (
          <div className="banner-telemetry-error" role="alert">
            {error}
          </div>
        ) : null}
        <StaleWarning serverIso={serverTs} />
      </Card>

      {msg?.kind === 'es_hk_v1' ? (
        <Panel title="ES HK" className="telemetry-grid">
          <EsHkPanel msg={msg} />
        </Panel>
      ) : null}
      {msg?.kind === 'parse_error' ? <ParseErrorPanel msg={msg} /> : null}
    </div>
  )
})
