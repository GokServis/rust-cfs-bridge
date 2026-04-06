import { observer } from 'mobx-react-lite'
import { useEffect, useState } from 'react'

export const StaleWarning = observer(function StaleWarning({ serverIso }: { serverIso: string | null }) {
  const [nowMs, setNowMs] = useState(() => performance.timeOrigin + performance.now())
  useEffect(() => {
    const id = window.setInterval(() => setNowMs(performance.timeOrigin + performance.now()), 5000)
    return () => window.clearInterval(id)
  }, [])
  if (!serverIso) {
    return <p className="telemetry__stale">No packets received yet.</p>
  }
  const ageSec = (nowMs - new Date(serverIso).getTime()) / 1000
  if (ageSec > 30) {
    return (
      <p className="telemetry__stale" role="status">
        Last update was {Math.round(ageSec)}s ago — link may be idle or cFS telemetry not enabled.
      </p>
    )
  }
  return null
})
