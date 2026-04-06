import { useCallback, useEffect, useState } from 'react'
import {
  buildNamedCommandJson,
  fetchCommands,
  sendCommandJson,
  type CommandMetadata,
} from './api'
import './App.css'

function payloadHint(p: CommandMetadata['payload']): string {
  if (p.kind === 'exact') {
    return `Override: ${p.bytes * 2} hex digits (${p.bytes} bytes), even length; or leave empty for the default payload.`
  }
  return `Payload length: ${p.min}–${p.max} bytes (hex).`
}

export default function App() {
  const [commands, setCommands] = useState<CommandMetadata[]>([])
  const [loadError, setLoadError] = useState<string | null>(null)
  const [selected, setSelected] = useState<string>('')
  const [sequenceCount, setSequenceCount] = useState(0)
  const [payloadHex, setPayloadHex] = useState('')
  const [status, setStatus] = useState<string | null>(null)
  const [sending, setSending] = useState(false)

  useEffect(() => {
    fetchCommands()
      .then((list) => {
        setCommands(list)
        if (list[0]) setSelected(list[0].name)
        setLoadError(null)
      })
      .catch((e: unknown) => {
        setLoadError(e instanceof Error ? e.message : 'Failed to load commands')
      })
  }, [])

  const current = commands.find((c) => c.name === selected)

  const onSend = useCallback(async () => {
    if (!selected) return
    setSending(true)
    setStatus(null)
    try {
      const json = buildNamedCommandJson(selected, sequenceCount, payloadHex)
      const res = await sendCommandJson(json)
      setStatus(`Sent ${res.bytes_sent} bytes on the wire (length ${res.wire_length}).`)
    } catch (e: unknown) {
      setStatus(e instanceof Error ? e.message : 'Send failed')
    } finally {
      setSending(false)
    }
  }, [selected, sequenceCount, payloadHex])

  return (
    <main className="app" aria-labelledby="app-title">
      <header className="app__header">
        <h1 id="app-title">cFS bridge</h1>
        <p className="app__lede" id="app-desc">
          Send dictionary commands as JSON; the server builds CCSDS packets and forwards them over UDP
          to CI_LAB.
        </p>
      </header>

      {loadError && (
        <div className="banner banner--error" role="alert">
          {loadError}
        </div>
      )}

      <section className="app__section" aria-labelledby="send-heading">
        <h2 id="send-heading" className="app__section-title">
          Send command
        </h2>

        <form
          className="form-card"
          onSubmit={(e) => {
            e.preventDefault()
            void onSend()
          }}
        >
          <div className="form-card__field">
            <label htmlFor="bridge-command">Command</label>
            <select
              id="bridge-command"
              value={selected}
              onChange={(e) => setSelected(e.target.value)}
              disabled={commands.length === 0}
              aria-describedby={current ? 'command-help' : undefined}
            >
              {commands.map((c) => (
                <option key={c.name} value={c.name}>
                  {c.title} ({c.name})
                </option>
              ))}
            </select>
          </div>

          {current && (
            <div id="command-help" className="help">
              <p className="help__intro">
                <strong>{current.title}.</strong> {current.description}
              </p>
              <dl className="help__ids">
                <div className="help__row">
                  <dt>CCSDS APID (on UDP wire)</dt>
                  <dd>
                    <code>0x{current.wire_apid.toString(16).padStart(3, '0')}</code>
                  </dd>
                </div>
                <div className="help__row">
                  <dt>Software Bus MsgId (after CI_LAB)</dt>
                  <dd>
                    <code>0x{current.software_bus_msg_id.toString(16).toUpperCase()}</code>
                  </dd>
                </div>
              </dl>
              <p className="help__payload">{payloadHint(current.payload)}</p>
            </div>
          )}

          <div className="form-card__field">
            <label htmlFor="sequence-count">Sequence count (0–16383)</label>
            <input
              id="sequence-count"
              type="number"
              min={0}
              max={0x3fff}
              value={sequenceCount}
              onChange={(e) => setSequenceCount(Number(e.target.value))}
              inputMode="numeric"
            />
          </div>

          <div className="form-card__field">
            <label htmlFor="payload-hex">Optional payload (hex digits, even length)</label>
            <input
              id="payload-hex"
              value={payloadHex}
              onChange={(e) => setPayloadHex(e.target.value)}
              placeholder="Leave empty for default"
              spellCheck={false}
              autoComplete="off"
              aria-describedby="payload-hint"
            />
            <span id="payload-hint" className="field-hint">
              Hex string only; odd length is rejected by the server.
            </span>
          </div>

          <button type="submit" className="btn-primary" disabled={sending || !selected}>
            {sending ? 'Sending…' : 'Send'}
          </button>
        </form>
      </section>

      {status && (
        <div
          className={`status ${status.startsWith('Sent') ? 'status--ok' : 'status--error'}`}
          role="status"
          aria-live="polite"
        >
          {status}
        </div>
      )}
    </main>
  )
}
