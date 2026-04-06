import { observer } from 'mobx-react-lite'

import type { AlertSeverityFilter, AlertStore } from '../../stores/alertStore'
import { Button } from '../ui/Button'
import { Panel } from '../ui/Panel'

import './AlertFeedPanel.css'

const SEVERITY_COLORS: Record<string, string> = {
  warn: 'var(--color-warn, #ff9800)',
  error: 'var(--color-error, #f44336)',
  critical: 'var(--color-critical, #9c27b0)',
}

const SEVERITY_LABELS: Record<string, string> = {
  warn: 'WARN',
  error: 'ERROR',
  critical: 'CRITICAL',
}

function formatAlertTime(iso: string): string {
  const t = new Date(iso).getTime()
  if (Number.isNaN(t)) return iso
  return new Date(t).toLocaleString()
}

export const AlertFeedPanel = observer(function AlertFeedPanel({ store }: { store: AlertStore }) {
  const {
    filteredAlerts,
    pagedAlerts,
    severityFilter,
    alertPageSize,
    effectiveAlertPageIndex,
    alertTotalPages,
  } = store

  return (
    <Panel title="Alerts" className="alert-feed">
      <div className="alert-feed__toolbar">
        <label className="alert-feed__field">
          <span className="alert-feed__label">Severity</span>
          <select
            value={severityFilter}
            onChange={(e) => store.setSeverityFilter(e.target.value as AlertSeverityFilter)}
            aria-label="Filter alerts by severity"
          >
            <option value="all">All</option>
            <option value="critical">Critical</option>
            <option value="error">Error</option>
            <option value="warn">Warning</option>
          </select>
        </label>
        <label className="alert-feed__field">
          <span className="alert-feed__label">Per page</span>
          <select
            value={alertPageSize}
            onChange={(e) => store.setAlertPageSize(Number(e.target.value))}
            aria-label="Alerts per page"
          >
            {[5, 7, 10, 15].map((n) => (
              <option key={n} value={n}>
                {n}
              </option>
            ))}
          </select>
        </label>
        <span className="alert-feed__toolbar-spacer" aria-hidden />
        <Button type="button" variant="ghost" onClick={() => store.clearAll()} disabled={store.alerts.length === 0}>
          Clear all
        </Button>
      </div>

      <div className="alert-feed__list" aria-live="polite" aria-relevant="additions removals">
        {pagedAlerts.length === 0 ? (
          <p className="alert-feed__empty">
            {store.alerts.length === 0 ? 'No alerts yet.' : 'No alerts match the current filter.'}
          </p>
        ) : (
          pagedAlerts.map((alert) => (
            <div
              key={alert.id}
              className={`alert-feed__card alert-feed__card--${alert.severity}`}
              role="status"
            >
              <span
                className="alert-feed__severity"
                style={{ color: SEVERITY_COLORS[alert.severity] ?? 'inherit' }}
              >
                {SEVERITY_LABELS[alert.severity] ?? alert.severity}
              </span>
              <div className="alert-feed__body">
                <span>{alert.message}</span>
                <time className="alert-feed__time" dateTime={alert.timestamp}>
                  {formatAlertTime(alert.timestamp)}
                </time>
              </div>
              <button
                type="button"
                onClick={() => store.dismissAlert(alert.id)}
                aria-label={`Dismiss alert: ${alert.message.slice(0, 80)}`}
                style={{
                  background: 'none',
                  border: 'none',
                  color: 'var(--text-muted)',
                  cursor: 'pointer',
                  padding: '0 0.25rem',
                  fontSize: '1.1rem',
                  lineHeight: 1,
                  flexShrink: 0,
                }}
              >
                ×
              </button>
            </div>
          ))
        )}
      </div>

      <p className="alert-feed__meta" aria-live="polite">
        Showing {filteredAlerts.length === 0 ? 0 : pagedAlerts.length} of {filteredAlerts.length} (buffered{' '}
        {store.alerts.length})
      </p>

      <div className="alert-feed__pager">
        <Button
          type="button"
          variant="ghost"
          disabled={effectiveAlertPageIndex <= 0}
          onClick={() => store.prevAlertPage()}
        >
          Previous
        </Button>
        <span style={{ fontSize: '0.8rem', opacity: 0.85 }}>
          Page {effectiveAlertPageIndex + 1} of {alertTotalPages}
        </span>
        <Button
          type="button"
          variant="ghost"
          disabled={effectiveAlertPageIndex >= alertTotalPages - 1}
          onClick={() => store.nextAlertPage()}
        >
          Next
        </Button>
      </div>
    </Panel>
  )
})
