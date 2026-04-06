import { Route, Routes } from 'react-router-dom'

import { CommandScreen } from './components/command/CommandScreen'
import { AppShell } from './components/layout/AppShell'
import { TelemetryScreen } from './components/telemetry/TelemetryScreen'

export default function App() {
  return (
    <AppShell>
      <Routes>
        <Route path="/" element={<CommandScreen />} />
        <Route path="/telemetry" element={<TelemetryScreen />} />
      </Routes>
    </AppShell>
  )
}
