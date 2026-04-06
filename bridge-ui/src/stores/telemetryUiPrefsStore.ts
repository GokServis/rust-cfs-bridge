import { makeAutoObservable } from 'mobx'

import type { KindFilter } from '../telemetryFiltering'

const STORAGE_KEY = 'bridge-ui-telemetry-filters'
const DEFAULT_PAGE_SIZE = 25

export interface TelemetryUiPrefsSnapshot {
  kindFilter: KindFilter
  apidFilter: string
  searchText: string
  pageSize: number
  hideParseError: boolean
}

function parseSnapshot(raw: string | null): TelemetryUiPrefsSnapshot | null {
  if (!raw) return null
  try {
    const j = JSON.parse(raw) as Partial<TelemetryUiPrefsSnapshot>
    const kind =
      j.kindFilter === 'all' ||
      j.kindFilter === 'es_hk_v1' ||
      j.kindFilter === 'to_lab_hk_v1' ||
      j.kindFilter === 'evs_long_event_v1' ||
      j.kindFilter === 'parse_error'
        ? j.kindFilter
        : null
    if (!kind) return null

    return {
      kindFilter: kind,
      apidFilter: typeof j.apidFilter === 'string' ? j.apidFilter : '',
      searchText: typeof j.searchText === 'string' ? j.searchText : '',
      pageSize: typeof j.pageSize === 'number' && Number.isFinite(j.pageSize) ? j.pageSize : DEFAULT_PAGE_SIZE,
      hideParseError: Boolean(j.hideParseError),
    }
  } catch {
    return null
  }
}

export class TelemetryUiPrefsStore {
  snapshot: TelemetryUiPrefsSnapshot = {
    kindFilter: 'all',
    apidFilter: '',
    searchText: '',
    pageSize: DEFAULT_PAGE_SIZE,
    hideParseError: false,
  }

  constructor() {
    makeAutoObservable(this)
    this.hydrate()
  }

  hydrate(): void {
    if (typeof window === 'undefined') return
    const s = parseSnapshot(localStorage.getItem(STORAGE_KEY))
    if (s) this.snapshot = s
  }

  setSnapshot(next: TelemetryUiPrefsSnapshot): void {
    this.snapshot = next
    this.persist()
  }

  private persist(): void {
    if (typeof localStorage === 'undefined') return
    localStorage.setItem(STORAGE_KEY, JSON.stringify(this.snapshot))
  }
}

