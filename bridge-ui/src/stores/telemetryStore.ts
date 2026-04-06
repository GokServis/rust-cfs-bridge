import { makeAutoObservable, runInAction } from 'mobx'

import type { TlmMessage } from '../telemetryTypes'

function telemetryWsUrl(): string {
  const { protocol, host } = window.location
  const wsProto = protocol === 'https:' ? 'wss:' : 'ws:'
  return `${wsProto}//${host}/api/tlm/ws`
}

export class TelemetryStore {
  connected = false
  lastReceivedAt: string | null = null
  lastMessage: TlmMessage | null = null
  error: string | null = null
  packetCount = 0

  private ws: WebSocket | null = null

  constructor() {
    makeAutoObservable(this)
  }

  connect(): void {
    if (typeof window === 'undefined') return
    this.disconnect()
    const socket = new WebSocket(telemetryWsUrl())
    this.ws = socket
    socket.onopen = () => {
      runInAction(() => {
        this.connected = true
        this.error = null
      })
    }
    socket.onclose = () => {
      runInAction(() => {
        this.connected = false
        this.ws = null
      })
    }
    socket.onerror = () => {
      runInAction(() => {
        this.error = 'WebSocket error'
      })
    }
    socket.onmessage = (ev: MessageEvent<string>) => {
      try {
        const msg = JSON.parse(ev.data) as TlmMessage
        runInAction(() => {
          this.lastMessage = msg
          this.lastReceivedAt = new Date().toISOString()
          this.packetCount += 1
        })
      } catch {
        runInAction(() => {
          this.error = 'Invalid telemetry JSON'
        })
      }
    }
  }

  disconnect(): void {
    this.ws?.close()
    this.ws = null
    runInAction(() => {
      this.connected = false
    })
  }
}
