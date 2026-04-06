import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi, afterEach } from 'vitest'
import App from './App'

const sampleCommands = [
  {
    name: 'CMD_HEARTBEAT',
    title: 'Heartbeat',
    description: 'Test.',
    wire_apid: 6,
    software_bus_msg_id: 0x18f0,
    payload: { kind: 'exact' as const, bytes: 3 },
  },
  {
    name: 'CMD_PING',
    title: 'Ping',
    description: 'Second command.',
    wire_apid: 7,
    software_bus_msg_id: 0x18f1,
    payload: { kind: 'exact' as const, bytes: 3 },
  },
]

describe('App', () => {
  const orig = globalThis.fetch

  afterEach(() => {
    globalThis.fetch = orig
    vi.restoreAllMocks()
  })

  it('loads commands and shows help text', async () => {
    globalThis.fetch = vi.fn((input: RequestInfo | URL) => {
      const url = typeof input === 'string' ? input : input.toString()
      if (url.includes('/api/commands')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(sampleCommands),
        } as Response)
      }
      return Promise.resolve({ ok: false, status: 404 } as Response)
    })

    render(<App />)
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /cFS bridge/i })).toBeInTheDocument()
    })
    expect(screen.getByRole('main')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: /Send command/i })).toBeInTheDocument()
    expect(await screen.findByText(/Software Bus MsgId/i)).toBeInTheDocument()
    const cmdSelect = screen.getByRole('combobox', { name: /Command/i })
    expect(cmdSelect.querySelectorAll('option')).toHaveLength(2)
  })

  it('sends command when Send is clicked', async () => {
    const user = userEvent.setup()
    globalThis.fetch = vi.fn((input: RequestInfo | URL, init?: RequestInit) => {
      const url = typeof input === 'string' ? input : input.toString()
      if (url.includes('/api/commands')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve(sampleCommands),
        } as Response)
      }
      if (url.includes('/api/send') && init?.method === 'POST') {
        return Promise.resolve({
          ok: true,
          text: () => Promise.resolve('{"bytes_sent":11,"wire_length":11}'),
        } as Response)
      }
      return Promise.resolve({ ok: false, status: 404 } as Response)
    })

    render(<App />)
    await screen.findByRole('button', { name: /Send/i })
    await user.click(screen.getByRole('button', { name: /^Send$/i }))
    await waitFor(() => {
      expect(screen.getByText(/Sent 11 bytes/i)).toBeInTheDocument()
    })
  })
})
