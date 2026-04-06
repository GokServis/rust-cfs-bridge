import { makeAutoObservable, runInAction } from 'mobx'

import type { TlmMessage } from '../telemetryTypes'
import {
  type KindFilter,
  type TlmEntry,
  filterEntries,
  pageSlice,
  totalPageCount,
} from '../telemetryFiltering'
import type { TelemetryUiPrefsStore } from './telemetryUiPrefsStore'

export const DEFAULT_TLM_BUFFER_CAP = 2000
export const DEFAULT_TLM_PAGE_SIZE = 25

function telemetryWsUrl(): string {
  const { protocol, host } = window.location
  const wsProto = protocol === 'https:' ? 'wss:' : 'ws:'
  return `${wsProto}//${host}/api/tlm/ws`
}

export class TelemetryStore {
  connected = false
  lastReceivedAt: string | null = null
  lastMessage: TlmMessage | null = null
  lastEsHk: Extract<TlmMessage, { kind: 'es_hk_v1' }> | null = null
  lastToLabHk: Extract<TlmMessage, { kind: 'to_lab_hk_v1' }> | null = null
  error: string | null = null
  packetCount = 0

  /** Newest-last ring buffer of parsed telemetry messages. */
  entries: TlmEntry[] = []
  maxEntries = DEFAULT_TLM_BUFFER_CAP

  kindFilter: KindFilter = 'all'
  apidFilter = ''
  searchText = ''
  hideParseError = false

  pageSize = DEFAULT_TLM_PAGE_SIZE
  pageIndex = 0

  private prefs?: TelemetryUiPrefsStore
  private ws: WebSocket | null = null
  private nextSeq = 1

  constructor(prefs?: TelemetryUiPrefsStore) {
    this.prefs = prefs
    makeAutoObservable(this)
    this.hydratePrefs()
  }

  private hydratePrefs(): void {
    const s = this.prefs?.snapshot
    if (!s) return
    this.kindFilter = s.kindFilter
    this.apidFilter = s.apidFilter
    this.searchText = s.searchText
    this.pageSize = Math.max(1, Math.min(500, Math.floor(s.pageSize)))
    this.hideParseError = s.hideParseError
  }

  private persistPrefs(): void {
    if (!this.prefs) return
    this.prefs.setSnapshot({
      kindFilter: this.kindFilter,
      apidFilter: this.apidFilter,
      searchText: this.searchText,
      pageSize: this.pageSize,
      hideParseError: this.hideParseError,
    })
  }

  get filteredEntries(): TlmEntry[] {
    return filterEntries(
      this.entries,
      this.kindFilter,
      this.apidFilter,
      this.searchText,
      this.hideParseError,
    )
  }

  get filteredCount(): number {
    return this.filteredEntries.length
  }

  get totalPages(): number {
    return totalPageCount(this.filteredCount, this.pageSize)
  }

  get effectivePageIndex(): number {
    return Math.min(this.pageIndex, Math.max(0, this.totalPages - 1))
  }

  get pagedEntries(): TlmEntry[] {
    return pageSlice(this.filteredEntries, this.pageSize, this.effectivePageIndex)
  }

  connect(): void {
    if (typeof window === 'undefined') return
    this.disconnect()
    const socket = new WebSocket(telemetryWsUrl())
    this.ws = socket
    socket.onopen = () => {
      runInAction(() => {
        this.connected = true
        this.error = null
      })
    }
    socket.onclose = () => {
      runInAction(() => {
        this.connected = false
        this.ws = null
      })
    }
    socket.onerror = () => {
      runInAction(() => {
        this.error = 'WebSocket error'
      })
    }
    socket.onmessage = (ev: MessageEvent<string>) => {
      try {
        const msg = JSON.parse(ev.data) as TlmMessage
        this.appendMessage(msg)
      } catch {
        runInAction(() => {
          this.error = 'Invalid telemetry JSON'
        })
      }
    }
  }

  /** Append one message (used by WebSocket and tests). */
  appendMessage(msg: TlmMessage): void {
    runInAction(() => {
      this.lastMessage = msg
      if (msg.kind === 'es_hk_v1') {
        this.lastEsHk = msg
      } else if (msg.kind === 'to_lab_hk_v1') {
        this.lastToLabHk = msg
      }
      this.lastReceivedAt = new Date().toISOString()
      this.packetCount += 1
      this.error = null
      this.entries.push({ seq: this.nextSeq++, message: msg })
      while (this.entries.length > this.maxEntries) {
        this.entries.shift()
      }
    })
  }

  disconnect(): void {
    this.ws?.close()
    this.ws = null
    runInAction(() => {
      this.connected = false
    })
  }

  setKindFilter(kind: KindFilter): void {
    runInAction(() => {
      this.kindFilter = kind
      this.pageIndex = 0
    })
    this.persistPrefs()
  }

  setApidFilter(value: string): void {
    runInAction(() => {
      this.apidFilter = value
      this.pageIndex = 0
    })
    this.persistPrefs()
  }

  setSearchText(value: string): void {
    runInAction(() => {
      this.searchText = value
      this.pageIndex = 0
    })
    this.persistPrefs()
  }

  setHideParseError(value: boolean): void {
    runInAction(() => {
      this.hideParseError = value
      this.pageIndex = 0
    })
    this.persistPrefs()
  }

  setPageSize(size: number): void {
    runInAction(() => {
      this.pageSize = Math.max(1, Math.min(500, Math.floor(size)))
      this.pageIndex = 0
    })
    this.persistPrefs()
  }

  goToPage(index: number): void {
    runInAction(() => {
      const max = Math.max(0, this.totalPages - 1)
      this.pageIndex = Math.min(Math.max(0, index), max)
    })
  }

  nextPage(): void {
    this.goToPage(this.effectivePageIndex + 1)
  }

  prevPage(): void {
    this.goToPage(this.effectivePageIndex - 1)
  }

  clearBuffer(): void {
    runInAction(() => {
      this.entries = []
      this.nextSeq = 1
      this.pageIndex = 0
    })
  }
}
