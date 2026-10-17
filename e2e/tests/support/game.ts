import { expect, type Page } from '@playwright/test'
import type { GameServerMessage } from '../../../web/src/game/schema'

type RoomFrame = Extract<GameServerMessage, { type: 'roomState' }>
type TestGame = {
  frames: GameServerMessage[]
  send: (command: Record<string, unknown>) => number
  close: () => void
}
type GameWindow = Window & { grenadiersTestGame?: TestGame }

export async function openGame(page: Page): Promise<void> {
  await page.goto('/')
  const authenticated = await page.evaluate(async () => {
    const response = await fetch('/api/v1/game/session', { method: 'POST' })
    const session: unknown = await response.json()
    if (response.status !== 201 || typeof session !== 'object' || session === null
      || !('token' in session) || typeof session.token !== 'string') {
      throw new Error('Session issuance failed')
    }
    const endpoint = new URL('/api/v1/game/ws', location.href)
    endpoint.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const socket = new WebSocket(endpoint)
    const frames: GameServerMessage[] = []
    let sequence = 0
    const first = new Promise<GameServerMessage>((resolve, reject) => {
      const timer = setTimeout(() => reject(new Error('Authentication timed out')), 10_000)
      socket.addEventListener('open', () => {
        socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 1, token: session.token }))
      }, { once: true })
      socket.addEventListener('error', () => {
        clearTimeout(timer)
        reject(new Error('Game connection failed'))
      }, { once: true })
      socket.addEventListener('message', (event: MessageEvent) => {
        if (typeof event.data !== 'string') throw new Error('Unexpected game frame')
        const message = JSON.parse(event.data) as GameServerMessage
        frames.push(message)
        if (frames.length > 256) frames.shift()
        clearTimeout(timer)
        resolve(message)
      })
    })
    const target = window as GameWindow
    target.grenadiersTestGame = {
      frames,
      send(command) {
        sequence += 1
        socket.send(JSON.stringify({ ...command, sequence }))
        return sequence
      },
      close() { socket.close() },
    }
    return first
  })
  expect(authenticated).toEqual({ type: 'authenticated', protocolVersion: 1 })
}

export async function sendGame(page: Page, command: Record<string, unknown>): Promise<number> {
  return page.evaluate((message) => {
    const driver = (window as GameWindow).grenadiersTestGame
    if (!driver) throw new Error('Game socket is not installed')
    return driver.send(message)
  }, command)
}

export async function gameFrames(page: Page): Promise<GameServerMessage[]> {
  return page.evaluate(() => (window as GameWindow).grenadiersTestGame?.frames ?? [])
}

export async function roomFrame(page: Page, afterRevision = 0): Promise<RoomFrame> {
  const latest = async (): Promise<RoomFrame | undefined> =>
    (await gameFrames(page)).filter((message) => message.type === 'roomState').at(-1)
  await expect.poll(async () => (await latest())?.room.revision ?? 0).toBeGreaterThan(afterRevision)
  const frame = await latest()
  if (!frame) throw new Error('No room state was received')
  return frame
}
