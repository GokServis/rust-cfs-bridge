import type { TlmMessage } from './telemetryTypes'

/** What to show in the log; `all` shows every buffered packet. */
export type KindFilter = 'all' | 'es_hk_v1' | 'to_lab_hk_v1' | 'evs_long_event_v1' | 'parse_error'

export interface TlmEntry {
  seq: number
  message: TlmMessage
}

export function apidOf(msg: TlmMessage): number | null {
  if (msg.kind === 'parse_error') {
    return msg.primary?.apid ?? null
  }
  if (msg.kind === 'es_hk_v1' || msg.kind === 'to_lab_hk_v1' || msg.kind === 'evs_long_event_v1') {
    return msg.primary.apid
  }
  return null
}

export function summaryLine(msg: TlmMessage): string {
  if (msg.kind === 'es_hk_v1') {
    return `ES HK · cmd ${msg.es_hk.command_counter} · heap free ${msg.es_hk.heap_bytes_free}`
  }
  if (msg.kind === 'to_lab_hk_v1') {
    return `TO_LAB HK · cmd ${msg.to_lab_hk.command_counter} · err ${msg.to_lab_hk.command_error_counter}`
  }
  if (msg.kind === 'evs_long_event_v1') {
    const p = msg.evs_long_event.packet_id
    return `${p.app_name} · EVS ${p.event_id} · type ${p.event_type} · ${msg.evs_long_event.message}`
  }
  return msg.message
}

export function matchesSearch(msg: TlmMessage, q: string): boolean {
  const t = q.trim()
  if (t === '') return true
  const lower = t.toLowerCase()
  if (JSON.stringify(msg).toLowerCase().includes(lower)) return true
  return false
}

export function filterEntries(
  entries: readonly TlmEntry[],
  kind: KindFilter,
  apidStr: string,
  searchText: string,
  hideParseError = false,
): TlmEntry[] {
  const trimmed = apidStr.trim()
  const apidNum = trimmed === '' ? null : Number.parseInt(trimmed, 10)
  const useApid = trimmed !== '' && !Number.isNaN(apidNum)

  return entries.filter((e) => {
    const m = e.message
    if (hideParseError && m.kind === 'parse_error') return false
    if (kind !== 'all' && m.kind !== kind) return false
    if (useApid) {
      const a = apidOf(m)
      if (a !== apidNum) return false
    }
    if (!matchesSearch(m, searchText)) return false
    return true
  })
}

export function pageSlice<T>(items: readonly T[], pageSize: number, pageIndex: number): T[] {
  const safeSize = Math.max(1, Math.floor(pageSize))
  const n = items.length
  const totalPages = Math.max(1, Math.ceil(n / safeSize))
  const idx = Math.min(Math.max(0, pageIndex), totalPages - 1)
  const start = idx * safeSize
  return items.slice(start, start + safeSize)
}

export function totalPageCount(itemCount: number, pageSize: number): number {
  const safeSize = Math.max(1, Math.floor(pageSize))
  return Math.max(1, Math.ceil(itemCount / safeSize))
}
