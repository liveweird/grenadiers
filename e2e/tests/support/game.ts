import { expect, type Page } from '@playwright/test'
import type { GameServerMessage } from '../../../web/src/game/v15/schema'

type RoomFrame = Extract<GameServerMessage, { type: 'roomState' | 'admitted' }>
type AdmissionFrame = Extract<GameServerMessage, { type: 'admitted' }>
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
        socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 15, token: session.token }))
      }, { once: true })
      socket.addEventListener('error', () => {
        clearTimeout(timer)
        reject(new Error('Game connection failed'))
      }, { once: true })
      socket.addEventListener('message', (event: MessageEvent) => {
        if (typeof event.data !== 'string') throw new Error('Unexpected game frame')
        const message = JSON.parse(event.data) as GameServerMessage
        // Personal capabilities must never enter trace-visible evaluate results.
        if (message.type === 'admitted') message.resumeToken = '[redacted]'
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
  expect(authenticated).toEqual({ type: 'authenticated', protocolVersion: 15 })
}

export async function sendGame(page: Page, command: Record<string, unknown>): Promise<number> {
  const message = command.type === 'createRoom' && command.configuration === undefined
    ? {
        ...command,
        configuration: command.mode && typeof command.mode === 'object'
          && 'type' in command.mode && command.mode.type === 'solo'
          ? {
              controllers: ['human', 'cpu'],
              cpuDifficulty: 'difficulty' in command.mode ? command.mode.difficulty : 'standard',
              board: { width: 15, height: 13 },
              environment: { stormCloud: true },
            }
          : {
              controllers: ['human', 'human'], cpuDifficulty: null,
              board: { width: 15, height: 13 },
              environment: { stormCloud: true },
            },
      }
    : command
  if ('mode' in message) delete message.mode
  return page.evaluate((message) => {
    const driver = (window as GameWindow).grenadiersTestGame
    if (!driver) throw new Error('Game socket is not installed')
    return driver.send(message)
  }, message)
}

export async function gameFrames(page: Page): Promise<GameServerMessage[]> {
  return page.evaluate(() => (window as GameWindow).grenadiersTestGame?.frames ?? [])
}

export async function roomFrame(
  page: Page,
  afterRevision = 0,
  matches: (frame: RoomFrame) => boolean = () => true,
): Promise<RoomFrame> {
  const latest = async (): Promise<RoomFrame | undefined> =>
    (await gameFrames(page)).filter((message): message is RoomFrame =>
      (message.type === 'roomState' || message.type === 'admitted')
      && message.room.revision > afterRevision && matches(message)).at(-1)
  await expect.poll(async () => Boolean(await latest())).toBe(true)
  const frame = await latest()
  if (!frame) throw new Error('No room state was received')
  return frame
}

export async function roomFrameAfter(
  page: Page,
  frameIndex: number,
  matches: (frame: RoomFrame) => boolean,
): Promise<RoomFrame> {
  const latest = async (): Promise<RoomFrame | undefined> =>
    (await gameFrames(page)).slice(frameIndex).filter((message): message is RoomFrame =>
      (message.type === 'roomState' || message.type === 'admitted') && matches(message)).at(-1)
  try {
    await expect.poll(async () => Boolean(await latest())).toBe(true)
  } catch (error) {
    const summaries = (await gameFrames(page)).slice(frameIndex).slice(-12).map((message) => {
      if (message.type !== 'roomState' && message.type !== 'admitted') return { type: message.type }
      return {
        type: message.type,
        revision: message.room.revision,
        tick: message.room.tick,
        phase: message.room.phase.type,
        seats: message.room.seats.map(({ occupied, connection }) => ({ occupied, connection })),
      }
    })
    throw new Error(`No matching room state after frame ${frameIndex}; observed ${JSON.stringify(summaries)}`,
      { cause: error })
  }
  const frame = await latest()
  if (!frame) throw new Error('No matching room state was received')
  return frame
}

export async function admissionFrame(page: Page): Promise<AdmissionFrame> {
  const latest = async () => (await gameFrames(page)).filter((message) => message.type === 'admitted').at(-1)
  await expect.poll(async () => Boolean(await latest())).toBe(true)
  const frame = await latest()
  if (!frame) throw new Error('No personal admission was received')
  return frame
}
