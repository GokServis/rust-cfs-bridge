import { waitFor } from '@testing-library/react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import { TelemetryStore } from './telemetryStore'

class MockWebSocket {
  static sockets: MockWebSocket[] = []
  onopen: (() => void) | null = null
  onmessage: ((ev: MessageEvent<string>) => void) | null = null
  onerror: (() => void) | null = null
  onclose: (() => void) | null = null
  constructor(url: string) {
    void url
    MockWebSocket.sockets.push(this)
    queueMicrotask(() => {
      this.onopen?.()
    })
  }
  close() {
    this.onclose?.()
  }
  triggerMessage(data: string) {
    this.onmessage?.({ data } as MessageEvent<string>)
  }
}

beforeEach(() => {
  MockWebSocket.sockets = []
  vi.stubGlobal('WebSocket', MockWebSocket as unknown as typeof WebSocket)
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('TelemetryStore', () => {
  it('connects and records message', async () => {
    const store = new TelemetryStore()
    store.connect()
    await waitFor(() => expect(store.connected).toBe(true))
    const payload = {
      kind: 'es_hk_v1',
      received_at: '2026-01-01T00:00:00.000Z',
      raw_len: 180,
      primary: { apid: 0, packet_type: 0, sequence_count: 0 },
      es_hk: {
        command_counter: 1,
        command_error_counter: 0,
        cfe_core_checksum: 0,
        cfe_version: [1, 2, 3, 4],
        osal_version: [0, 0, 0, 0],
        psp_version: [0, 0, 0, 0],
        syslog_bytes_used: 0,
        syslog_size: 0,
        syslog_entries: 0,
        syslog_mode: 0,
        registered_core_apps: 0,
        registered_external_apps: 0,
        registered_tasks: 0,
        registered_libs: 0,
        reset_type: 0,
        reset_subtype: 0,
        processor_resets: 0,
        max_processor_resets: 0,
        boot_source: 0,
        perf_state: 0,
        perf_mode: 0,
        perf_trigger_count: 0,
        heap_bytes_free: 0,
        heap_blocks_free: 0,
        heap_max_block_size: 0,
      },
    }
    MockWebSocket.sockets[0].triggerMessage(JSON.stringify(payload))
    await waitFor(() => expect(store.packetCount).toBe(1))
    expect(store.lastMessage?.kind).toBe('es_hk_v1')
    expect(store.error).toBeNull()
    store.disconnect()
  })

  it('sets error on invalid JSON', async () => {
    const store = new TelemetryStore()
    store.connect()
    await waitFor(() => expect(store.connected).toBe(true))
    MockWebSocket.sockets[0].triggerMessage('not-json')
    await waitFor(() => expect(store.error).toBe('Invalid telemetry JSON'))
    store.disconnect()
  })
})
