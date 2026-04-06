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
    return `Payload must be exactly ${p.bytes} bytes (hex).`
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
    <div className="app">
      <header>
        <h1>cFS bridge</h1>
        <p className="lede">
          Send dictionary commands as JSON; the server builds CCSDS packets and forwards them over UDP
          to CI_LAB.
        </p>
      </header>

      {loadError && <p className="error">{loadError}</p>}

      <section className="panel">
        <label>
          Command
          <select
            value={selected}
            onChange={(e) => setSelected(e.target.value)}
            disabled={commands.length === 0}
          >
            {commands.map((c) => (
              <option key={c.name} value={c.name}>
                {c.title} ({c.name})
              </option>
            ))}
          </select>
        </label>

        {current && (
          <p className="help">
            <strong>{current.title}.</strong> {current.description} Software Bus MsgId:{' '}
            <code>0x{current.software_bus_msg_id.toString(16).toUpperCase()}</code>, wire APID:{' '}
            <code>0x{current.wire_apid.toString(16).padStart(3, '0')}</code>. {payloadHint(current.payload)}
          </p>
        )}

        <label>
          Sequence count (0–16383)
          <input
            type="number"
            min={0}
            max={0x3fff}
            value={sequenceCount}
            onChange={(e) => setSequenceCount(Number(e.target.value))}
          />
        </label>

        <label>
          Optional payload (hex digits, even length)
          <input
            value={payloadHex}
            onChange={(e) => setPayloadHex(e.target.value)}
            placeholder="leave empty for default"
            spellCheck={false}
          />
        </label>

        <button type="button" onClick={() => void onSend()} disabled={sending || !selected}>
          {sending ? 'Sending…' : 'Send'}
        </button>
      </section>

      {status && <p className={status.startsWith('Sent') ? 'ok' : 'error'}>{status}</p>}
    </div>
  )
}
