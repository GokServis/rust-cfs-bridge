import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import {
  buildNamedCommandJson,
  fetchCommands,
  sendCommandJson,
} from './api'

describe('buildNamedCommandJson', () => {
  it('omits payload when empty', () => {
    const j = buildNamedCommandJson('CMD_HEARTBEAT', 0, '')
    expect(j).toBe('{"command":"CMD_HEARTBEAT","sequence_count":0}')
  })

  it('includes hex payload when set', () => {
    const j = buildNamedCommandJson('CMD_HEARTBEAT', 2, '  aabbcc  ')
    expect(JSON.parse(j)).toEqual({
      command: 'CMD_HEARTBEAT',
      sequence_count: 2,
      payload: 'aabbcc',
    })
  })
})

describe('fetchCommands', () => {
  const orig = globalThis.fetch

  afterEach(() => {
    globalThis.fetch = orig
  })

  it('returns JSON array on success', async () => {
    globalThis.fetch = vi.fn(() =>
      Promise.resolve({
        ok: true,
        json: () => Promise.resolve([{ name: 'X' }]),
      } as Response),
    )
    const list = await fetchCommands()
    expect(list).toEqual([{ name: 'X' }])
  })

  it('throws on HTTP error', async () => {
    globalThis.fetch = vi.fn(() =>
      Promise.resolve({
        ok: false,
        status: 500,
      } as Response),
    )
    await expect(fetchCommands()).rejects.toThrow('500')
  })
})

describe('sendCommandJson', () => {
  const orig = globalThis.fetch

  afterEach(() => {
    globalThis.fetch = orig
  })

  beforeEach(() => {
    globalThis.fetch = vi.fn()
  })

  it('returns result on success', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue({
      ok: true,
      text: () => Promise.resolve('{"bytes_sent":11,"wire_length":11}'),
    } as Response)

    const r = await sendCommandJson('{}')
    expect(r.bytes_sent).toBe(11)
  })

  it('throws server error message', async () => {
    vi.mocked(globalThis.fetch).mockResolvedValue({
      ok: false,
      status: 400,
      text: () => Promise.resolve('{"error":"bad"}'),
    } as Response)

    await expect(sendCommandJson('{}')).rejects.toThrow('bad')
  })
})
