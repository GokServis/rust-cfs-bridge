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
})
