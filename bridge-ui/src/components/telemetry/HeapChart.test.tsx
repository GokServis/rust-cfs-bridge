import { render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'

import { HeapChart } from './HeapChart'
import { formatChartBytes, formatChartTime } from './telemetryChartFormat'

vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="responsive-container">{children}</div>
  ),
  LineChart: ({ children }: { children: React.ReactNode }) => (
    <div data-testid="line-chart">{children}</div>
  ),
  Line: () => null,
  XAxis: () => null,
  YAxis: () => null,
  Tooltip: () => null,
  CartesianGrid: () => null,
  Legend: () => null,
}))

describe('HeapChart', () => {
  it('renders the chart container', () => {
    render(<HeapChart data={[]} />)
    expect(screen.getByTestId('line-chart')).toBeInTheDocument()
  })

  it('renders with populated data without crashing', () => {
    const data = [
      { t: 1_700_000_000_000, heap_bytes_free: 500_000, heap_max_block_size: 200_000 },
      { t: 1_700_000_001_000, heap_bytes_free: 480_000, heap_max_block_size: 190_000 },
    ]
    render(<HeapChart data={data} />)
    expect(screen.getByTestId('line-chart')).toBeInTheDocument()
  })

  it('shows a placeholder message when data is empty', () => {
    render(<HeapChart data={[]} />)
    expect(screen.getByText(/no heap data/i)).toBeInTheDocument()
  })
})

describe('formatChartBytes', () => {
  it('formats bytes less than 1 KB as raw bytes', () => {
    expect(formatChartBytes(512)).toBe('512 B')
  })

  it('formats values >= 1 KB as KB', () => {
    expect(formatChartBytes(2048)).toBe('2 KB')
  })

  it('formats values >= 1 MB as MB', () => {
    expect(formatChartBytes(2 * 1_048_576)).toBe('2.0 MB')
  })
})

describe('formatChartTime', () => {
  it('returns a non-empty time string for a valid timestamp', () => {
    const result = formatChartTime(1_700_000_000_000)
    expect(typeof result).toBe('string')
    expect(result.length).toBeGreaterThan(0)
  })
})
