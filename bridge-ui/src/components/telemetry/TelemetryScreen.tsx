import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'

import { useStore } from '../../stores/useStore'

import { TelemetryLogTable } from './TelemetryLogTable'
import { TelemetryOverview } from './TelemetryOverview'

import './TelemetryScreen.css'

export const TelemetryScreen = observer(function TelemetryScreen() {
  const { telemetry } = useStore()

  useEffect(() => {
    telemetry.connect()
    return () => telemetry.disconnect()
  }, [telemetry])

  return (
    <div className="telemetry-screen">
      <div className="telemetry-screen__overview">
        <TelemetryOverview store={telemetry} />
      </div>
      <div className="telemetry-screen__log">
        <TelemetryLogTable store={telemetry} />
      </div>
    </div>
  )
})
