import { observer } from 'mobx-react-lite'
import { useEffect } from 'react'

import type { CommandMetadata } from '../../api'
import { Button } from '../ui/Button'
import { Card } from '../ui/Card'
import { Panel } from '../ui/Panel'
import { useStore } from '../../stores/useStore'
import { CommandHistoryLog } from './CommandHistoryLog'

import './CommandScreen.css'

function payloadHint(p: CommandMetadata['payload']): string {
  if (p.kind === 'exact') {
    return `Override: ${p.bytes * 2} hex digits (${p.bytes} bytes), even length; or leave empty for the default payload.`
  }
  return `Payload length: ${p.min}–${p.max} bytes (hex).`
}

export const CommandScreen = observer(function CommandScreen() {
  const { command: c } = useStore()

  useEffect(() => {
    void c.load()
  }, [c])

  const current = c.commands.find((x) => x.name === c.selected)

  return (
    <div className="command-screen">
      <div className="command-screen__hero">
        <p className="command-screen__title">Uplink</p>
        <p className="command-screen__lede" id="command-desc">
          Send dictionary commands as JSON; the server builds CCSDS packets and forwards them over UDP to
          CI_LAB.
        </p>
      </div>

      {c.loadError ? (
        <div className="banner-error" role="alert">
          {c.loadError}
        </div>
      ) : null}

      <Card>
        <h2 className="command-screen__title" id="send-heading">
          Send command
        </h2>
        <form
          className="command-form"
          aria-describedby="command-desc"
          onSubmit={(e) => {
            e.preventDefault()
            void c.send()
          }}
        >
          <div className="command-form__field">
            <label htmlFor="bridge-command">Command</label>
            <select
              id="bridge-command"
              value={c.selected}
              onChange={(e) => c.setSelected(e.target.value)}
              disabled={c.commands.length === 0}
              aria-describedby={current ? 'command-help' : undefined}
            >
              {c.commands.map((cmd) => (
                <option key={cmd.name} value={cmd.name}>
                  {cmd.title} ({cmd.name})
                </option>
              ))}
            </select>
          </div>

          {current ? (
            <div id="command-help" className="command-help">
              <p className="command-help__intro">
                <strong>{current.title}.</strong> {current.description}
              </p>
              <dl className="command-help__ids">
                <div className="command-help__row">
                  <dt>CCSDS APID (on UDP wire)</dt>
                  <dd>
                    <code>0x{current.wire_apid.toString(16).padStart(3, '0')}</code>
                  </dd>
                </div>
                <div className="command-help__row">
                  <dt>Software Bus MsgId (after CI_LAB)</dt>
                  <dd>
                    <code>0x{current.software_bus_msg_id.toString(16).toUpperCase()}</code>
                  </dd>
                </div>
              </dl>
              <p className="command-help__payload">{payloadHint(current.payload)}</p>
            </div>
          ) : null}

          <div className="command-form__field">
            <label htmlFor="sequence-count">Sequence count (0–16383)</label>
            <input
              id="sequence-count"
              type="number"
              min={0}
              max={0x3fff}
              value={c.sequenceCount}
              onChange={(e) => c.setSequenceCount(Number(e.target.value))}
              inputMode="numeric"
            />
          </div>

          <div className="command-form__field">
            <label htmlFor="payload-hex">Optional payload (hex digits, even length)</label>
            <input
              id="payload-hex"
              value={c.payloadHex}
              onChange={(e) => c.setPayloadHex(e.target.value)}
              placeholder="Leave empty for default"
              spellCheck={false}
              autoComplete="off"
              aria-describedby="payload-hint"
            />
            <span id="payload-hint" className="field-hint">
              Hex string only; odd length is rejected by the server.
            </span>
          </div>

          <Button type="submit" disabled={c.sending || !c.selected}>
            {c.sending ? 'Sending…' : 'Send'}
          </Button>
        </form>
      </Card>

      {c.status ? (
        <div
          className={`command-status ${c.status.startsWith('Sent') ? 'command-status--ok' : 'command-status--err'}`}
          role="status"
          aria-live="polite"
        >
          {c.status}
        </div>
      ) : null}

      <Panel title="Command History">
        <CommandHistoryLog history={c.history} onClear={() => c.clearHistory()} />
      </Panel>
    </div>
  )
})
