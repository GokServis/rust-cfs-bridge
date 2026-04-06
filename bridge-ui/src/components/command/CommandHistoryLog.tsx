import type { CommandHistoryEntry } from '../../stores/commandStore'

interface Props {
  history: CommandHistoryEntry[]
  onClear: () => void
}

export function CommandHistoryLog({ history, onClear }: Props) {
  return (
    <div>
      <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '0.5rem' }}>
        <span style={{ fontSize: '0.85rem', opacity: 0.7 }}>{history.length} command{history.length !== 1 ? 's' : ''} sent</span>
        {history.length > 0 && (
          <button type="button" onClick={onClear} style={{ fontSize: '0.75rem' }}>
            Clear history
          </button>
        )}
      </div>

      {history.length === 0 ? (
        <p style={{ textAlign: 'center', opacity: 0.5, fontSize: '0.85rem' }}>No commands sent yet</p>
      ) : (
        <div style={{ overflowX: 'auto' }}>
          <table style={{ width: '100%', borderCollapse: 'collapse', fontSize: '0.8rem' }}>
            <thead>
              <tr>
                <th style={{ textAlign: 'left', padding: '0.25rem 0.5rem' }}>Time</th>
                <th style={{ textAlign: 'left', padding: '0.25rem 0.5rem' }}>Command</th>
                <th style={{ textAlign: 'right', padding: '0.25rem 0.5rem' }}>Seq</th>
                <th style={{ textAlign: 'center', padding: '0.25rem 0.5rem' }}>Status</th>
                <th style={{ textAlign: 'right', padding: '0.25rem 0.5rem' }}>Bytes</th>
              </tr>
            </thead>
            <tbody>
              {[...history].reverse().map((entry, i) => (
                <tr
                  key={i}
                  className={entry.status === 'rejected' ? 'history-row history-row--rejected' : 'history-row'}
                >
                  <td style={{ padding: '0.2rem 0.5rem', opacity: 0.7 }}>
                    {new Date(entry.sentAt).toLocaleTimeString()}
                  </td>
                  <td style={{ padding: '0.2rem 0.5rem', fontFamily: 'monospace' }}>{entry.name}</td>
                  <td style={{ padding: '0.2rem 0.5rem', textAlign: 'right' }}>{entry.sequenceCount}</td>
                  <td
                    style={{
                      padding: '0.2rem 0.5rem',
                      textAlign: 'center',
                      color: entry.status === 'rejected' ? 'var(--color-error, #f44336)' : 'var(--color-ok, #4caf50)',
                    }}
                  >
                    {entry.status}
                  </td>
                  <td style={{ padding: '0.2rem 0.5rem', textAlign: 'right' }}>
                    {entry.status === 'sent' ? entry.wireLength : '—'}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      )}
    </div>
  )
}
