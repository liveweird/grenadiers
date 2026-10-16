import { describe, expect, it, vi } from 'vitest'
import { RealtimeClient, realtimeUrl } from './client'

class SocketStub {
  static readonly OPEN = 1
  readonly listeners = new Map<string, Array<(event: MessageEvent) => void>>()
  readyState = SocketStub.OPEN
  sent: string[] = []
  closed = false

  readonly url: string

  constructor(url: string) { this.url = url }

  addEventListener(name: string, listener: (event: MessageEvent) => void): void {
    const listeners = this.listeners.get(name) ?? []
    listeners.push(listener)
    this.listeners.set(name, listeners)
  }

  send(message: string): void { this.sent.push(message) }
  close(): void { this.closed = true }
  emit(name: string, data: unknown = undefined): void {
    for (const listener of this.listeners.get(name) ?? []) listener({ data } as MessageEvent)
  }
}

describe('RealtimeClient', () => {
  it('validates messages, sends pings and closes cleanly', () => {
    const messages = vi.fn()
    const failures = vi.fn()
    const sockets: SocketStub[] = []
    vi.stubGlobal('WebSocket', class extends SocketStub {
      constructor(url: string) { super(url); sockets.push(this) }
    })
    Object.assign(WebSocket, { OPEN: SocketStub.OPEN })

    const client = new RealtimeClient(messages, failures)
    expect(client.ping(1)).toBe(false)
    client.connect('ws://example.test/api/v1/ws')
    sockets[0].emit('message', JSON.stringify({ type: 'welcome', protocolVersion: 1, maxPlayers: 2 }))
    expect(messages).toHaveBeenCalledWith({ type: 'welcome', protocolVersion: 1, maxPlayers: 2 })
    sockets[0].emit('message', JSON.stringify({ type: 'error', code: 'invalidMessage', message: 'Invalid client message' }))
    expect(messages).toHaveBeenLastCalledWith({ type: 'error', code: 'invalidMessage', message: 'Invalid client message' })
    expect(client.ping(7)).toBe(true)
    expect(sockets[0].sent).toEqual(['{"type":"ping","sequence":7}'])

    sockets[0].emit('message', '{bad json')
    expect(sockets[0].closed).toBe(true)
    sockets[0].emit('message', JSON.stringify({ type: 'pong', sequence: -1 }))
    sockets[0].emit('message', new Blob())
    expect(failures).toHaveBeenCalledTimes(1)
    client.disconnect()
    expect(sockets[0].closed).toBe(true)
    sockets[0].emit('close')
    expect(client.isConnected()).toBe(false)
  })

  it('reports transport failures and derives secure URLs', () => {
    const failure = vi.fn()
    const sockets: SocketStub[] = []
    vi.stubGlobal('WebSocket', class extends SocketStub {
      constructor(url: string) { super(url); sockets.push(this) }
    })
    const client = new RealtimeClient(vi.fn(), failure)
    client.connect('ws://example.test/api/v1/ws')
    sockets[0].emit('error')
    expect(failure).toHaveBeenCalledWith('connection')
    expect(sockets[0].closed).toBe(true)
    expect(realtimeUrl({ protocol: 'https:', host: 'game.test' } as Location)).toBe('wss://game.test/api/v1/ws')
  })

  it('reports a current remote close and ignores stale socket callbacks', () => {
    const messages = vi.fn()
    const failures = vi.fn()
    const sockets: SocketStub[] = []
    vi.stubGlobal('WebSocket', class extends SocketStub {
      constructor(url: string) { super(url); sockets.push(this) }
    })
    const client = new RealtimeClient(messages, failures)
    client.connect('ws://example.test/first')
    client.connect('ws://example.test/second')
    sockets[0].emit('message', JSON.stringify({ type: 'welcome', protocolVersion: 1, maxPlayers: 2 }))
    sockets[0].emit('close')
    expect(messages).not.toHaveBeenCalled()
    expect(failures).not.toHaveBeenCalled()
    sockets[1].emit('close')
    expect(failures).toHaveBeenCalledWith('connection')
  })
})
