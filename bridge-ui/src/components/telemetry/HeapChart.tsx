import {
  CartesianGrid,
  Legend,
  Line,
  LineChart,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'

import { formatChartBytes, formatChartTime } from './telemetryChartFormat'

export interface HeapDataPoint {
  t: number
  heap_bytes_free: number
  heap_max_block_size: number
}

interface Props {
  data: HeapDataPoint[]
}

export function HeapChart({ data }: Props) {
  return (
    <div>
      <div style={{ height: 180 }}>
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={data} margin={{ top: 4, right: 8, bottom: 4, left: 8 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border, #444)" />
            <XAxis
              dataKey="t"
              tickFormatter={formatChartTime}
              tick={{ fontSize: 10 }}
              minTickGap={40}
            />
            <YAxis tickFormatter={formatChartBytes} tick={{ fontSize: 10 }} width={52} />
            <Tooltip
              labelFormatter={(v) => formatChartTime(Number(v))}
              formatter={(value) => [formatChartBytes(Number(value ?? 0))]}
            />
            <Legend iconSize={10} wrapperStyle={{ fontSize: '0.75rem' }} />
            <Line
              type="monotone"
              dataKey="heap_bytes_free"
              name="Free heap"
              stroke="var(--color-ok, #4caf50)"
              dot={false}
              isAnimationActive={false}
            />
            <Line
              type="monotone"
              dataKey="heap_max_block_size"
              name="Max block"
              stroke="var(--color-warn, #ff9800)"
              strokeDasharray="4 2"
              dot={false}
              isAnimationActive={false}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
      {data.length === 0 && (
        <p style={{ textAlign: 'center', opacity: 0.5, fontSize: '0.8rem', margin: '0.25rem 0' }}>
          No heap data yet
        </p>
      )}
    </div>
  )
}
