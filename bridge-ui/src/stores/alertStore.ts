import { makeAutoObservable, runInAction } from 'mobx'

import type { TlmMessage } from '../telemetryTypes'
import { pageSlice, totalPageCount } from '../telemetryFiltering'

export type AlertSeverity = 'warn' | 'error' | 'critical'

export type AlertSeverityFilter = 'all' | AlertSeverity

export interface Alert {
  id: number
  severity: AlertSeverity
  message: string
  timestamp: string
}

/** Minimum free heap bytes before a WARN alert fires. */
const HEAP_WARN_THRESHOLD = 128 * 1024

export const DEFAULT_ALERT_PAGE_SIZE = 7

let nextId = 1

export class AlertStore {
  alerts: Alert[] = []

  severityFilter: AlertSeverityFilter = 'all'
  alertPageSize = DEFAULT_ALERT_PAGE_SIZE
  alertPageIndex = 0

  private lastEsErrorCount = 0
  private lastProcessorResets = 0
  private lastToLabErrorCount = 0

  constructor() {
    makeAutoObservable(this)
  }

  /** Newest-first (higher `id` = more recently created). */
  get alertsNewestFirst(): Alert[] {
    return [...this.alerts].sort((a, b) => b.id - a.id)
  }

  get filteredAlerts(): Alert[] {
    if (this.severityFilter === 'all') return this.alertsNewestFirst
    return this.alertsNewestFirst.filter(a => a.severity === this.severityFilter)
  }

  get alertTotalPages(): number {
    return totalPageCount(this.filteredAlerts.length, this.alertPageSize)
  }

  get effectiveAlertPageIndex(): number {
    return Math.min(this.alertPageIndex, Math.max(0, this.alertTotalPages - 1))
  }

  get pagedAlerts(): Alert[] {
    return pageSlice(this.filteredAlerts, this.alertPageSize, this.effectiveAlertPageIndex)
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
      this.alertPageIndex = 0
    })
  }

  setSeverityFilter(filter: AlertSeverityFilter): void {
    runInAction(() => {
      this.severityFilter = filter
      this.alertPageIndex = 0
    })
  }

  setAlertPageSize(size: number): void {
    runInAction(() => {
      this.alertPageSize = Math.max(1, Math.min(50, Math.floor(size)))
      this.alertPageIndex = 0
    })
  }

  goToAlertPage(index: number): void {
    runInAction(() => {
      const max = Math.max(0, this.alertTotalPages - 1)
      this.alertPageIndex = Math.min(Math.max(0, index), max)
    })
  }

  nextAlertPage(): void {
    this.goToAlertPage(this.effectiveAlertPageIndex + 1)
  }

  prevAlertPage(): void {
    this.goToAlertPage(this.effectiveAlertPageIndex - 1)
  }

  private push(severity: AlertSeverity, message: string): void {
    runInAction(() => {
      this.alerts.push({ id: nextId++, severity, message, timestamp: new Date().toISOString() })
    })
  }
}
