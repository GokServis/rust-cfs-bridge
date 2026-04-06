import { makeAutoObservable, runInAction } from 'mobx'

import type { TlmMessage } from '../telemetryTypes'

export type AlertSeverity = 'warn' | 'error' | 'critical'

export interface Alert {
  id: number
  severity: AlertSeverity
  message: string
  timestamp: string
}

/** Minimum free heap bytes before a WARN alert fires. */
const HEAP_WARN_THRESHOLD = 128 * 1024

let nextId = 1

export class AlertStore {
  alerts: Alert[] = []

  private lastEsErrorCount = 0
  private lastProcessorResets = 0
  private lastToLabErrorCount = 0

  constructor() {
    makeAutoObservable(this)
  }

  evaluate(msg: TlmMessage): void {
    if (msg.kind === 'es_hk_v1') {
      const hk = msg.es_hk

      if (hk.command_error_counter > this.lastEsErrorCount) {
        this.push('error', `ES command error counter incremented to ${hk.command_error_counter}`)
      }
      this.lastEsErrorCount = hk.command_error_counter

      if (hk.processor_resets > this.lastProcessorResets) {
        this.push('critical', `Processor reset detected! Count: ${hk.processor_resets}`)
      }
      this.lastProcessorResets = hk.processor_resets

      if (hk.heap_bytes_free < HEAP_WARN_THRESHOLD) {
        this.push('warn', `Heap low: ${(hk.heap_bytes_free / 1024).toFixed(0)} KB free`)
      }
    }

    if (msg.kind === 'to_lab_hk_v1') {
      const hk = msg.to_lab_hk
      if (hk.command_error_counter > this.lastToLabErrorCount) {
        this.push('warn', `TO_LAB command error counter incremented to ${hk.command_error_counter}`)
      }
      this.lastToLabErrorCount = hk.command_error_counter
    }

    if (msg.kind === 'command_ack' && msg.result === 'rejected') {
      this.push('error', `Command ${msg.name} (seq ${msg.sequence_count}) was REJECTED by cFS`)
    }
  }

  dismissAlert(id: number): void {
    runInAction(() => {
      this.alerts = this.alerts.filter(a => a.id !== id)
    })
  }

  clearAll(): void {
    runInAction(() => {
      this.alerts = []
    })
  }

  private push(severity: AlertSeverity, message: string): void {
    runInAction(() => {
      this.alerts.push({ id: nextId++, severity, message, timestamp: new Date().toISOString() })
    })
  }
}
