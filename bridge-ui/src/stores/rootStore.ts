import { CommandStore } from './commandStore'
import { TelemetryStore } from './telemetryStore'
import { ThemeStore } from './themeStore'

export class RootStore {
  theme: ThemeStore
  command: CommandStore
  telemetry: TelemetryStore

  constructor() {
    this.theme = new ThemeStore()
    this.command = new CommandStore()
    this.telemetry = new TelemetryStore()
  }
}
