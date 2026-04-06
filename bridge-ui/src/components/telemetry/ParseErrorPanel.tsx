import type { TlmMessage } from '../../telemetryTypes'

import { Panel } from '../ui/Panel'

export function ParseErrorPanel({ msg }: { msg: Extract<TlmMessage, { kind: 'parse_error' }> }) {
  return (
    <Panel title="Parse note" className="telemetry-parse-error">
      <p>{msg.message}</p>
      <pre className="telemetry-hex">{msg.hex_preview}</pre>
    </Panel>
  )
}
