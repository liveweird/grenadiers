import type { Page } from '@playwright/test'
import type { Direction, GameServerMessage, Seat } from '../../../web/src/game/v9/schema'

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
  moveTakeover: (direction: Direction) => void
  stopTakeover: () => void
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
    let takeoverBinding: { socket: WebSocket; roomId: string; roundId: number; sequence: number } | null = null
    let sharedLeak = false

    function publicAdmission(message: Admission, stableCapability: boolean): PublicAdmission {
      return {
        roomId: message.room.roomId, seat: message.youSeat,
        actionFloor: message.lastActionId, tick: message.room.tick,
        hasState: message.state !== null,
        positions: message.state?.actors.flatMap((actor) => actor.type === 'visible'
          ? [{ seat: actor.seat, ...actor.position }] : []) ?? [],
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
        let stage: 'created' | 'open' | 'authenticated' | 'resumeSent' = 'created'
        let publicMessageCount = 0
        let lastPublicMessageType = 'none'
        const startedAt = performance.now()
        const failure = (message: string): Error => new Error(
          `${message} (stage=${stage} readyState=${socket.readyState} `
          + `publicMessages=${publicMessageCount} lastPublicMessage=${lastPublicMessageType} `
          + `elapsedMs=${Math.max(0, Math.round(performance.now() - startedAt))})`,
        )
        const timer = setTimeout(() => {
          const error = failure('Private resume probe timed out')
          cleanup()
          socket.close()
          reject(error)
        }, 10_000)
        const fail = (): void => {
          cleanup()
          reject(failure('Private resume probe closed before admission'))
        }
        function cleanup(): void {
          clearTimeout(timer)
          socket.removeEventListener('message', onMessage)
          socket.removeEventListener('close', fail)
          socket.removeEventListener('error', fail)
        }
        socket.addEventListener('error', fail, { once: true })
        socket.addEventListener('close', fail, { once: true })
        socket.addEventListener('open', () => {
          stage = 'open'
          socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 9, token: session.token }))
        }, { once: true })
        const onMessage = (event: MessageEvent): void => {
          const message = JSON.parse(String(event.data)) as GameServerMessage
          publicMessageCount += 1
          lastPublicMessageType = ['authenticated', 'admitted', 'error', 'roomState', 'snapshot'].includes(message.type)
            ? message.type : 'other'
          if (message.type === 'authenticated') {
            stage = 'authenticated'
            const altered = `${capability.token[0] === '0' ? '1' : '0'}${capability.token.slice(1)}`
            socket.send(JSON.stringify({
              type: 'resumeRoom', sequence: 1,
              roomId: kind === 'wrongRoom' ? 'f'.repeat(32) : capability.roomId,
              resumeToken: kind === 'wrongCapability' ? altered : capability.token,
            }))
            stage = 'resumeSent'
          } else if (message.type === 'admitted' || message.type === 'error') {
            cleanup()
            if (message.type === 'error') {
              socket.close()
              resolve(message.code)
            } else {
              if (kind === undefined) {
                takeoverBinding = {
                  socket, roomId: message.room.roomId, roundId: message.room.roundId, sequence: 2,
                }
              }
              resolve(publicAdmission(message, message.resumeToken === capability.token))
            }
          }
        }
        socket.addEventListener('message', onMessage)
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
      moveTakeover(direction) {
        if (!takeoverBinding || takeoverBinding.socket.readyState !== NativeSocket.OPEN) {
          throw new Error('No admitted takeover socket is available')
        }
        const binding = takeoverBinding
        binding.socket.send(JSON.stringify({
          type: 'movement', sequence: binding.sequence++, roomId: binding.roomId,
          roundId: binding.roundId, direction,
        }))
      },
      stopTakeover() {
        if (!takeoverBinding || takeoverBinding.socket.readyState !== NativeSocket.OPEN) {
          throw new Error('No admitted takeover socket is available')
        }
        const binding = takeoverBinding
        binding.socket.send(JSON.stringify({
          type: 'movement', sequence: binding.sequence++, roomId: binding.roomId,
          roundId: binding.roundId, direction: null,
        }))
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

export async function moveTakeover(page: Page, direction: Direction): Promise<void> {
  await page.evaluate((value) => {
    const probe = (window as ProbeWindow).grenadiersAcceptanceTransport
    if (!probe) throw new Error('Transport probe is not installed')
    probe.moveTakeover(value)
  }, direction)
}

export async function stopTakeover(page: Page): Promise<void> {
  await page.evaluate(() => {
    const probe = (window as ProbeWindow).grenadiersAcceptanceTransport
    if (!probe) throw new Error('Transport probe is not installed')
    probe.stopTakeover()
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
