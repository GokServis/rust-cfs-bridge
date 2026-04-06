import { AlertStore } from './alertStore'
import { CommandStore } from './commandStore'
import { TelemetryStore } from './telemetryStore'
import { ThemeStore } from './themeStore'
import { TelemetryUiPrefsStore } from './telemetryUiPrefsStore'

export class RootStore {
  theme: ThemeStore
  command: CommandStore
  telemetry: TelemetryStore
  telemetryUiPrefs: TelemetryUiPrefsStore
  alerts: AlertStore

  constructor() {
    this.theme = new ThemeStore()
    this.command = new CommandStore()
    this.telemetryUiPrefs = new TelemetryUiPrefsStore()
    this.alerts = new AlertStore()
    this.telemetry = new TelemetryStore(this.telemetryUiPrefs, this.alerts)
  }
}
