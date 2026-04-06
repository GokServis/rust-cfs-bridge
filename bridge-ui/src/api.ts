/** Types aligned with `rust_bridge::CommandMetadata` JSON from `GET /api/commands`. */

export type PayloadConstraint =
  | { kind: 'exact'; bytes: number }
  | { kind: 'range'; min: number; max: number }

export interface CommandMetadata {
  name: string
  title: string
  description: string
  wire_apid: number
  software_bus_msg_id: number
  payload: PayloadConstraint
}

export interface SendResult {
  bytes_sent: number
  wire_length: number
}

export async function setToLabOutputEnabled(enabled: boolean): Promise<SendResult> {
  const r = await fetch(enabled ? '/api/to_lab/output/enable' : '/api/to_lab/output/disable', {
    method: 'POST',
  })
  const text = await r.text()
  if (!r.ok) {
    let msg = `Request failed (${r.status})`
    try {
      const j = JSON.parse(text) as { error?: string }
      if (j.error) msg = j.error
    } catch {
      /* ignore */
    }
    throw new Error(msg)
  }
  return JSON.parse(text) as SendResult
}

export async function fetchCommands(): Promise<CommandMetadata[]> {
  const r = await fetch('/api/commands')
  if (!r.ok) {
    throw new Error(`Failed to load commands (${r.status})`)
  }
  return r.json() as Promise<CommandMetadata[]>
}

export async function sendCommandJson(jsonBody: string): Promise<SendResult> {
  const r = await fetch('/api/send', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: jsonBody,
  })
  const text = await r.text()
  if (!r.ok) {
    let msg = `Send failed (${r.status})`
    try {
      const j = JSON.parse(text) as { error?: string }
      if (j.error) msg = j.error
    } catch {
      /* ignore */
    }
    throw new Error(msg)
  }
  return JSON.parse(text) as SendResult
}

/** Builds the JSON body for a named dictionary command (matches Rust `SpaceCommand::from_json`). */
export function buildNamedCommandJson(
  command: string,
  sequenceCount: number,
  payloadHex: string,
): string {
  const trimmed = payloadHex.trim()
  if (trimmed.length > 0) {
    return JSON.stringify({
      command,
      sequence_count: sequenceCount,
      payload: trimmed,
    })
  }
  return JSON.stringify({ command, sequence_count: sequenceCount })
}
