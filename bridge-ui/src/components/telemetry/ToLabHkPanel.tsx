import type { TlmMessage } from '../../telemetryTypes'

export function ToLabHkPanel({ msg }: { msg: Extract<TlmMessage, { kind: 'to_lab_hk_v1' }> }) {
  const { to_lab_hk: hk } = msg
  return (
    <dl className="telemetry-dl">
      <div className="telemetry-dl__row">
        <dt>Command counter</dt>
        <dd>{hk.command_counter}</dd>
      </div>
      <div className="telemetry-dl__row">
        <dt>Command error counter</dt>
        <dd>{hk.command_error_counter}</dd>
      </div>
    </dl>
  )
}
