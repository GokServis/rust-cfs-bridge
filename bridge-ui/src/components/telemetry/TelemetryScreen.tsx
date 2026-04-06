import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'

import { useStore } from '../../stores/useStore'

import { TelemetryOverview } from './TelemetryOverview'

export const TelemetryScreen = observer(function TelemetryScreen() {
  const { telemetry } = useStore()

  useEffect(() => {
    telemetry.connect()
    return () => telemetry.disconnect()
  }, [telemetry])

  return <TelemetryOverview store={telemetry} />
})
