import './StatusBadge.css'

export function StatusBadge({ ok, children }: { ok: boolean; children: string }) {
  return (
    <span className={`ui-badge ${ok ? 'ui-badge--ok' : 'ui-badge--bad'}`}>
      {ok ? <span className="ui-badge__pulse" aria-hidden /> : null}
      {children}
    </span>
  )
}
