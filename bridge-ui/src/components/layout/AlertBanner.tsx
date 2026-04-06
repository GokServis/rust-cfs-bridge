import { observer } from 'mobx-react-lite'

import type { AlertStore } from '../../stores/alertStore'

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

export const AlertBanner = observer(function AlertBanner({ store }: { store: AlertStore }) {
  if (store.alerts.length === 0) return null

  return (
    <div
      role="alert"
      aria-live="assertive"
      style={{ display: 'flex', flexDirection: 'column', gap: '2px' }}
    >
      {store.alerts.map(alert => (
        <div
          key={alert.id}
          style={{
            display: 'flex',
            alignItems: 'center',
            gap: '0.5rem',
            padding: '0.35rem 0.75rem',
            background: SEVERITY_COLORS[alert.severity] ?? '#555',
            color: '#fff',
            fontSize: '0.82rem',
          }}
        >
          <strong>{SEVERITY_LABELS[alert.severity] ?? alert.severity}</strong>
          <span style={{ flex: 1 }}>{alert.message}</span>
          <button
            type="button"
            onClick={() => store.dismissAlert(alert.id)}
            aria-label="Dismiss alert"
            style={{
              background: 'none',
              border: 'none',
              color: '#fff',
              cursor: 'pointer',
              padding: '0 0.25rem',
              fontSize: '1rem',
              lineHeight: 1,
            }}
          >
            ×
          </button>
        </div>
      ))}
    </div>
  )
})
