import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'

import * as api from '../api'

import { CommandStore } from './commandStore'

describe('CommandStore', () => {
  beforeEach(() => {
    vi.spyOn(api, 'fetchCommands').mockResolvedValue([
      {
        name: 'CMD_HEARTBEAT',
        title: 'Heartbeat',
        description: 'x',
        wire_apid: 6,
        software_bus_msg_id: 0x18f0,
        payload: { kind: 'exact', bytes: 3 },
      },
    ])
    vi.spyOn(api, 'sendCommandJson').mockResolvedValue({ bytes_sent: 11, wire_length: 11 })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  it('load populates commands', async () => {
    const store = new CommandStore()
    await store.load()
    expect(store.commands).toHaveLength(1)
    expect(store.selected).toBe('CMD_HEARTBEAT')
    expect(store.loadError).toBeNull()
  })

  it('load sets loadError on failure', async () => {
    vi.mocked(api.fetchCommands).mockRejectedValueOnce(new Error('network'))
    const store = new CommandStore()
    await store.load()
    expect(store.loadError).toBe('network')
  })

  it('send updates status', async () => {
    const store = new CommandStore()
    await store.load()
    await store.send()
    expect(store.status).toMatch(/Sent 11 bytes/)
  })

  it('setters update fields', () => {
    const store = new CommandStore()
    store.setSelected('CMD_PING')
    store.setSequenceCount(2)
    store.setPayloadHex('00')
    expect(store.selected).toBe('CMD_PING')
    expect(store.sequenceCount).toBe(2)
    expect(store.payloadHex).toBe('00')
  })

  describe('command history', () => {
    it('history is empty initially', () => {
      const store = new CommandStore()
      expect(store.history).toEqual([])
    })

    it('appends a "sent" entry after a successful send', async () => {
      const store = new CommandStore()
      await store.load()
      await store.send()
      expect(store.history).toHaveLength(1)
      const entry = store.history[0]
      expect(entry.name).toBe('CMD_HEARTBEAT')
      expect(entry.status).toBe('sent')
      expect(entry.wireLength).toBe(11)
      expect(typeof entry.sentAt).toBe('string')
      expect(entry.sequenceCount).toBe(0)
    })

    it('appends a "rejected" entry when send throws', async () => {
      vi.mocked(api.sendCommandJson).mockRejectedValueOnce(new Error('bad command'))
      const store = new CommandStore()
      await store.load()
      await store.send()
      expect(store.history).toHaveLength(1)
      expect(store.history[0].status).toBe('rejected')
    })

    it('accumulates multiple sends in order', async () => {
      const store = new CommandStore()
      await store.load()
      await store.send()
      await store.send()
      expect(store.history).toHaveLength(2)
      expect(store.history[0].sequenceCount).toBe(0)
      expect(store.history[1].sequenceCount).toBe(0)
    })

    it('clearHistory empties the list', async () => {
      const store = new CommandStore()
      await store.load()
      await store.send()
      store.clearHistory()
      expect(store.history).toHaveLength(0)
    })
  })
})
