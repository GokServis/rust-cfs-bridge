import { makeAutoObservable, runInAction } from 'mobx'

import {
  buildNamedCommandJson,
  fetchCommands,
  sendCommandJson,
  type CommandMetadata,
} from '../api'

export interface CommandHistoryEntry {
  name: string
  sentAt: string
  sequenceCount: number
  /** 'sent' = server accepted; 'rejected' = network/validation error. */
  status: 'sent' | 'rejected'
  wireLength: number
}

export class CommandStore {
  commands: CommandMetadata[] = []
  loadError: string | null = null
  selected = ''
  sequenceCount = 0
  payloadHex = ''
  status: string | null = null
  sending = false
  history: CommandHistoryEntry[] = []

  constructor() {
    makeAutoObservable(this)
  }

  async load(): Promise<void> {
    try {
      const list = await fetchCommands()
      runInAction(() => {
        this.commands = list
        this.loadError = null
        if (list[0]) this.selected = list[0].name
      })
    } catch (e: unknown) {
      runInAction(() => {
        this.loadError = e instanceof Error ? e.message : 'Failed to load commands'
      })
    }
  }

  setSelected(name: string): void {
    this.selected = name
  }

  setSequenceCount(n: number): void {
    this.sequenceCount = n
  }

  setPayloadHex(s: string): void {
    this.payloadHex = s
  }

  async send(): Promise<void> {
    if (!this.selected) return
    const name = this.selected
    const seq = this.sequenceCount
    this.sending = true
    this.status = null
    try {
      const json = buildNamedCommandJson(name, seq, this.payloadHex)
      const res = await sendCommandJson(json)
      runInAction(() => {
        this.status = `Sent ${res.bytes_sent} bytes on the wire (length ${res.wire_length}).`
        this.history.push({
          name,
          sentAt: new Date().toISOString(),
          sequenceCount: seq,
          status: 'sent',
          wireLength: res.wire_length,
        })
      })
    } catch (e: unknown) {
      runInAction(() => {
        this.status = e instanceof Error ? e.message : 'Send failed'
        this.history.push({
          name,
          sentAt: new Date().toISOString(),
          sequenceCount: seq,
          status: 'rejected',
          wireLength: 0,
        })
      })
    } finally {
      runInAction(() => {
        this.sending = false
      })
    }
  }

  clearHistory(): void {
    runInAction(() => {
      this.history = []
    })
  }
}
