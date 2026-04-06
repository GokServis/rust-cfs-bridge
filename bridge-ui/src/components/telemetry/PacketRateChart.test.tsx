import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { PacketRateChart, formatTime } from './PacketRateChart'

// Recharts uses ResizeObserver and SVG which are not fully supported in happy-dom.
// Mock the library so tests focus on component logic, not charting internals.
vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
  BarChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="bar-chart">{children}</div>
  ),
  Bar: () => null,
  XAxis: () => null,
  YAxis: () => null,
  Tooltip: () => null,
  CartesianGrid: () => null,
}))

describe('PacketRateChart', () => {
  it('renders the chart container', () => {
    render(<PacketRateChart data={[]} />)
    expect(screen.getByTestId('bar-chart')).toBeInTheDocument()
  })

  it('renders with populated data without crashing', () => {
    const data = [
      { t: 1_700_000_000_000, rate: 3 },
      { t: 1_700_000_001_000, rate: 1 },
    ]
    render(<PacketRateChart data={data} />)
    expect(screen.getByTestId('bar-chart')).toBeInTheDocument()
  })

  it('shows a placeholder message when data is empty', () => {
    render(<PacketRateChart data={[]} />)
    expect(screen.getByText(/no packet data/i)).toBeInTheDocument()
  })
})

describe('formatTime (PacketRateChart)', () => {
  it('returns a non-empty time string for a valid timestamp', () => {
    const result = formatTime(1_700_000_000_000)
    expect(typeof result).toBe('string')
    expect(result.length).toBeGreaterThan(0)
  })
})
