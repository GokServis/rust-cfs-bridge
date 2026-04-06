import { makeAutoObservable, runInAction } from 'mobx'

import {
  buildNamedCommandJson,
  fetchCommands,
  sendCommandJson,
  type CommandMetadata,
} from '../api'

export class CommandStore {
  commands: CommandMetadata[] = []
  loadError: string | null = null
  selected = ''
  sequenceCount = 0
  payloadHex = ''
  status: string | null = null
  sending = false

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
    this.sending = true
    this.status = null
    try {
      const json = buildNamedCommandJson(this.selected, this.sequenceCount, this.payloadHex)
      const res = await sendCommandJson(json)
      runInAction(() => {
        this.status = `Sent ${res.bytes_sent} bytes on the wire (length ${res.wire_length}).`
      })
    } catch (e: unknown) {
      runInAction(() => {
        this.status = e instanceof Error ? e.message : 'Send failed'
      })
    } finally {
      runInAction(() => {
        this.sending = false
      })
    }
  }
}
