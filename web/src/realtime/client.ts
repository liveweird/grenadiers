import Ajv from 'ajv'
import realtimeSchema from '../../../api/realtime.schema.json'
import type { ClientMessage, ServerMessage } from './schema'

type Listener = (message: ServerMessage) => void
type FailureListener = (message: string) => void

const ajv = new Ajv({ allErrors: true })
const validateServerMessage = ajv.compile<ServerMessage>(realtimeSchema)

export function realtimeUrl(location: Location = window.location): string {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${location.host}/api/v1/ws`
}

export class RealtimeClient {
  private socket: WebSocket | undefined
  private readonly onMessage: Listener
  private readonly onFailure: FailureListener

  constructor(onMessage: Listener, onFailure: FailureListener) {
    this.onMessage = onMessage
    this.onFailure = onFailure
  }

  connect(url = realtimeUrl()): void {
    this.disconnect()
    const socket = new WebSocket(url)
    this.socket = socket
    socket.addEventListener('message', (event) => this.receive(socket, event.data))
    socket.addEventListener('error', () => this.fail(socket, 'connection'))
    socket.addEventListener('close', () => this.fail(socket, 'connection'))
  }

  disconnect(): void {
    this.socket?.close()
    this.socket = undefined
  }

  ping(sequence: number): boolean {
    if (this.socket?.readyState !== WebSocket.OPEN) return false
    const message: ClientMessage = { type: 'ping', sequence }
    this.socket.send(JSON.stringify(message))
    return true
  }

  isConnected(): boolean {
    return this.socket?.readyState === WebSocket.OPEN
  }

  private receive(socket: WebSocket, raw: unknown): void {
    if (this.socket !== socket) return
    if (typeof raw !== 'string') {
      this.fail(socket, 'invalidMessage')
      return
    }
    try {
      const message: unknown = JSON.parse(raw)
      if (!validateServerMessage(message)) {
        this.fail(socket, 'invalidMessage')
        return
      }
      this.onMessage(message)
    } catch {
      this.fail(socket, 'invalidMessage')
    }
  }

  private fail(socket: WebSocket, failure: string): void {
    if (this.socket !== socket) return
    this.socket = undefined
    socket.close()
    this.onFailure(failure)
  }
}
