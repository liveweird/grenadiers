import type { Page } from '@playwright/test'
import type { GameServerMessage, Seat } from '../../../web/src/game/v5/schema'

type Admission = Extract<GameServerMessage, { type: 'admitted' }>
export type PublicAdmission = {
  roomId: string
  seat: Seat
  actionFloor: number | null
  tick: number
  hasState: boolean
  positions: { seat: Seat; x: number; y: number }[]
  bombIds: number[]
  stableCapability: boolean
}

type TransportProbe = {
  disconnect: () => void
  baseline: () => PublicAdmission | null
  takeover: () => Promise<PublicAdmission>
  invalidResume: (kind: 'wrongRoom' | 'wrongCapability') => Promise<string>
  leakedSharedFields: () => boolean
}
type ProbeWindow = Window & { grenadiersAcceptanceTransport?: TransportProbe }

export async function installTransportProbe(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const NativeSocket = window.WebSocket
    let appSocket: WebSocket | null = null
    let credential: { roomId: string; seat: Seat; token: string } | null = null
    let admitted: PublicAdmission | null = null
    let sharedLeak = false

    function publicAdmission(message: Admission, stableCapability: boolean): PublicAdmission {
      return {
        roomId: message.room.roomId, seat: message.youSeat,
        actionFloor: message.lastBombActionId, tick: message.room.tick,
        hasState: message.state !== null,
        positions: message.state?.actors.map((actor) => ({ seat: actor.seat, ...actor.position })) ?? [],
        bombIds: message.state?.bombs.map((bomb) => bomb.id) ?? [], stableCapability,
      }
    }

    window.WebSocket = class extends NativeSocket {
      constructor(url: string | URL, protocols?: string | string[]) {
        super(url, protocols)
        if (!new URL(String(url), location.href).pathname.endsWith('/game/ws')) return
        this.addEventListener('open', (event) => { appSocket = event.currentTarget as WebSocket }, { once: true })
        this.addEventListener('message', (event: MessageEvent) => {
          if (typeof event.data !== 'string') return
          const message = JSON.parse(event.data) as GameServerMessage
          if (message.type === 'admitted') {
            const stable = credential === null || credential.roomId !== message.room.roomId
              || credential.token === message.resumeToken
            credential = { roomId: message.room.roomId, seat: message.youSeat, token: message.resumeToken }
            admitted = publicAdmission(message, stable)
          }
          if (message.type === 'roomState' || message.type === 'snapshot') {
            const encoded = JSON.stringify(message)
            sharedLeak ||= (credential !== null && encoded.includes(credential.token))
              || ['resumeToken', 'generationSeed', 'sessionToken', 'seed', 'roundSeed', 'generationRecord']
                .some((key) => encoded.includes(`"${key}"`))
          }
        })
      }
    }

    async function resume(kind?: 'wrongRoom' | 'wrongCapability'): Promise<PublicAdmission | string> {
      if (!credential) throw new Error('No private seat capability is available in this tab')
      const capability = credential
      const response = await fetch('/api/v1/game/session', { method: 'POST' })
      if (response.status !== 201) throw new Error('Fresh takeover session failed')
      const session = await response.json() as { token: string }
      const endpoint = new URL('/api/v1/game/ws', location.href)
      endpoint.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
      const socket = new NativeSocket(endpoint)
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => {
          socket.close()
          reject(new Error('Private resume probe timed out'))
        }, 10_000)
        const fail = (): void => {
          clearTimeout(timer)
          reject(new Error('Private resume probe closed before admission'))
        }
        socket.addEventListener('error', fail, { once: true })
        socket.addEventListener('close', fail, { once: true })
        socket.addEventListener('open', () => {
          socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 5, token: session.token }))
        }, { once: true })
        socket.addEventListener('message', (event: MessageEvent) => {
          const message = JSON.parse(String(event.data)) as GameServerMessage
          if (message.type === 'authenticated') {
            const altered = `${capability.token[0] === '0' ? '1' : '0'}${capability.token.slice(1)}`
            socket.send(JSON.stringify({
              type: 'resumeRoom', sequence: 1,
              roomId: kind === 'wrongRoom' ? 'f'.repeat(32) : capability.roomId,
              resumeToken: kind === 'wrongCapability' ? altered : capability.token,
            }))
          } else if (message.type === 'admitted' || message.type === 'error') {
            clearTimeout(timer)
            socket.removeEventListener('close', fail)
            socket.removeEventListener('error', fail)
            if (message.type === 'error') {
              socket.close()
              resolve(message.code)
            } else {
              resolve(publicAdmission(message, message.resumeToken === capability.token))
            }
          }
        })
      })
    }

    ;(window as ProbeWindow).grenadiersAcceptanceTransport = {
      disconnect() { appSocket?.close(1000, 'Acceptance transport loss') },
      baseline() { return admitted },
      async takeover() {
        const result = await resume()
        if (typeof result === 'string') throw new Error(`Takeover rejected: ${result}`)
        return result
      },
      async invalidResume(kind) {
        const result = await resume(kind)
        if (typeof result !== 'string') throw new Error('An invalid capability was unexpectedly admitted')
        return result
      },
      leakedSharedFields() { return sharedLeak },
    }
  })
}

export async function disconnectTransport(page: Page): Promise<void> {
  await page.evaluate(() => {
    const probe = (window as ProbeWindow).grenadiersAcceptanceTransport
    if (!probe) throw new Error('Transport probe is not installed')
    probe.disconnect()
  })
}

export async function admissionBaseline(page: Page): Promise<PublicAdmission | null> {
  return page.evaluate(() => (window as ProbeWindow).grenadiersAcceptanceTransport?.baseline() ?? null)
}

export async function takeover(page: Page): Promise<PublicAdmission> {
  return page.evaluate(async () => {
    const probe = (window as ProbeWindow).grenadiersAcceptanceTransport
    if (!probe) throw new Error('Transport probe is not installed')
    return probe.takeover()
  })
}

export async function invalidResume(page: Page, kind: 'wrongRoom' | 'wrongCapability'): Promise<string> {
  return page.evaluate(async (value) => {
    const probe = (window as ProbeWindow).grenadiersAcceptanceTransport
    if (!probe) throw new Error('Transport probe is not installed')
    return probe.invalidResume(value)
  }, kind)
}

export async function sharedFieldsArePrivate(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const probe = (window as ProbeWindow).grenadiersAcceptanceTransport
    if (!probe) throw new Error('Transport probe is not installed')
    return !probe.leakedSharedFields()
  })
}
