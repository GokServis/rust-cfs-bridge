import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, it, expect, vi, afterEach } from 'vitest'
import { MemoryRouter } from 'react-router-dom'

import App from './App'
import { RootStore } from './stores/rootStore'
import { StoreProvider } from './stores/StoreProvider'

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

function renderApp(path = '/') {
  const store = new RootStore()
  return render(
    <StoreProvider store={store}>
      <MemoryRouter initialEntries={[path]}>
        <App />
      </MemoryRouter>
    </StoreProvider>,
  )
}

describe('App', () => {
  const orig = globalThis.fetch

  afterEach(() => {
    globalThis.fetch = orig
    vi.restoreAllMocks()
  })

  it('loads commands and shows uplink UI', async () => {
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

    renderApp('/')
    await waitFor(() => {
      expect(screen.getByRole('heading', { name: /cFS bridge/i })).toBeInTheDocument()
    })
    expect(screen.getByRole('navigation', { name: /Primary/i })).toBeInTheDocument()
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

    renderApp('/')
    await screen.findByRole('button', { name: /Send/i })
    await user.click(screen.getByRole('button', { name: /^Send$/i }))
    await waitFor(() => {
      expect(screen.getByText(/Sent 11 bytes/i)).toBeInTheDocument()
    })
  })

  it('shows telemetry screen on route', async () => {
    globalThis.fetch = vi.fn(() => Promise.resolve({ ok: false, status: 404 } as Response))
    renderApp('/telemetry')
    expect(await screen.findByRole('heading', { name: /Live telemetry/i })).toBeInTheDocument()
  })
})
