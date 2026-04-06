import { render, screen, fireEvent } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import type { CommandHistoryEntry } from '../../stores/commandStore'
import { CommandHistoryLog } from './CommandHistoryLog'

const sentEntry: CommandHistoryEntry = {
  name: 'CMD_HEARTBEAT',
  sentAt: '2026-01-01T12:00:00.000Z',
  sequenceCount: 1,
  status: 'sent',
  wireLength: 11,
}

const rejectedEntry: CommandHistoryEntry = {
  name: 'CMD_PING',
  sentAt: '2026-01-01T12:00:01.000Z',
  sequenceCount: 2,
  status: 'rejected',
  wireLength: 0,
}

describe('CommandHistoryLog', () => {
  it('shows empty state when history is empty', () => {
    render(<CommandHistoryLog history={[]} onClear={vi.fn()} />)
    expect(screen.getByText(/no commands sent/i)).toBeInTheDocument()
  })

  it('renders a row per history entry', () => {
    render(<CommandHistoryLog history={[sentEntry, rejectedEntry]} onClear={vi.fn()} />)
    expect(screen.getByText('CMD_HEARTBEAT')).toBeInTheDocument()
    expect(screen.getByText('CMD_PING')).toBeInTheDocument()
  })

  it('highlights rejected entries', () => {
    render(<CommandHistoryLog history={[rejectedEntry]} onClear={vi.fn()} />)
    const row = screen.getByText('CMD_PING').closest('tr')
    expect(row?.className).toMatch(/rejected/)
  })

  it('calls onClear when clear button is clicked', () => {
    const onClear = vi.fn()
    render(<CommandHistoryLog history={[sentEntry]} onClear={onClear} />)
    fireEvent.click(screen.getByRole('button', { name: /clear/i }))
    expect(onClear).toHaveBeenCalledOnce()
  })

  it('displays wire length for sent entries', () => {
    render(<CommandHistoryLog history={[sentEntry]} onClear={vi.fn()} />)
    expect(screen.getByText('11')).toBeInTheDocument()
  })
})
