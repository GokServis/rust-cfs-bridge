import {
  Bar,
  BarChart,
  CartesianGrid,
  ResponsiveContainer,
  Tooltip,
  XAxis,
  YAxis,
} from 'recharts'

export interface PacketRateBucket {
  t: number
  rate: number
}

interface Props {
  data: PacketRateBucket[]
}

export function formatTime(t: number): string {
  return new Date(t).toLocaleTimeString([], { hour: '2-digit', minute: '2-digit', second: '2-digit' })
}

export function PacketRateChart({ data }: Props) {
  return (
    <div>
      <div style={{ height: 160 }}>
        <ResponsiveContainer width="100%" height="100%">
          <BarChart data={data} margin={{ top: 4, right: 8, bottom: 4, left: 0 }}>
            <CartesianGrid strokeDasharray="3 3" stroke="var(--color-border, #444)" />
            <XAxis
              dataKey="t"
              tickFormatter={formatTime}
              tick={{ fontSize: 10 }}
              minTickGap={40}
            />
            <YAxis allowDecimals={false} tick={{ fontSize: 10 }} width={28} />
            <Tooltip
              labelFormatter={(v) => formatTime(Number(v))}
              formatter={(value) => [Number(value ?? 0), 'pkt/s']}
            />
            <Bar dataKey="rate" fill="var(--color-accent, #4a9eff)" isAnimationActive={false} />
          </BarChart>
        </ResponsiveContainer>
      </div>
      {data.length === 0 && (
        <p style={{ textAlign: 'center', opacity: 0.5, fontSize: '0.8rem', margin: '0.25rem 0' }}>
          No packet data yet
        </p>
      )}
    </div>
  )
}
