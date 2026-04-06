import { observer } from 'mobx-react-lite'
import { Route, Routes } from 'react-router-dom'

import { CommandScreen } from './components/command/CommandScreen'
import { AppShell } from './components/layout/AppShell'
import { HealthDashboard } from './components/telemetry/HealthDashboard'
import { TelemetryScreen } from './components/telemetry/TelemetryScreen'
import { useStore } from './stores/useStore'

const HealthScreen = observer(function HealthScreen() {
  const { telemetry, alerts } = useStore()
  return <HealthDashboard telemetry={telemetry} alerts={alerts} />
})

export default function App() {
  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<CommandScreen />} />
        <Route path="/telemetry" element={<TelemetryScreen />} />
        <Route path="/dashboard" element={<HealthScreen />} />
      </Routes>
    </AppShell>
  )
}
