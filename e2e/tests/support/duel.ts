import { expect, type Page } from '@playwright/test'
import type { ActorView, Direction, GameServerMessage, Seat } from '../../../web/src/game/v8/schema'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
export type ObservedGame = {
  frames: GameServerMessage[]
  commands: { type: string; action?: 'bomb' | 'equipment'; actionId?: number; direction?: Direction | null }[]
  sockets: number
}

export function observeGame(page: Page): ObservedGame {
  const observed: ObservedGame = { frames: [], commands: [], sockets: 0 }
  page.on('websocket', (socket) => {
    if (!new URL(socket.url()).pathname.endsWith('/game/ws')) return
    observed.sockets += 1
    socket.on('framesent', ({ payload }) => {
      const command = JSON.parse(payload.toString()) as {
        type: string
        actionId?: number
        action?: unknown
        direction?: unknown
      }
      if (command.type !== 'authenticate') {
        let direction: Direction | null | undefined
        if (['north', 'east', 'south', 'west'].includes(String(command.direction))) {
          direction = command.direction as Direction
        } else if (command.direction === null) {
          direction = null
        }
        const summary: ObservedGame['commands'][number] = {
          type: command.type,
          actionId: command.actionId,
        }
        if (command.type === 'movement') summary.direction = direction
        if (command.type === 'manualAction' && (command.action === 'bomb' || command.action === 'equipment')) {
          summary.action = command.action
        }
        observed.commands.push(summary)
      }
    })
    socket.on('framereceived', ({ payload }) => {
      const frame = JSON.parse(payload.toString()) as GameServerMessage
      if (frame.type === 'admitted') frame.resumeToken = '[redacted]'
      observed.frames.push(frame)
      if (observed.frames.length > 512) observed.frames.shift()
    })
  })
  return observed
}

export function latestSnapshot(game: ObservedGame): Snapshot | undefined {
  return game.frames.filter((frame) => frame.type === 'snapshot').at(-1)
}

export function visibleActor(snapshot: Snapshot | undefined, seat: Seat) {
  return snapshot?.state.actors.find((actor): actor is Extract<ActorView, { type: 'visible' }> =>
    actor.seat === seat && actor.type === 'visible')
}

export async function expectPaintedCanvas(page: Page): Promise<void> {
  const canvas = page.locator('.game-arena canvas')
  await expect(canvas).toBeVisible()
  const screenshot = await canvas.screenshot()
  // Decode a compositor screenshot: reading the WebGL drawing buffer after presentation
  // can legitimately return cleared pixels when preserveDrawingBuffer is false.
  const colors = await page.evaluate(async (png) => {
    const image = new Image()
    image.src = `data:image/png;base64,${png}`
    await image.decode()
    const sample = document.createElement('canvas')
    sample.width = image.width
    sample.height = image.height
    const context = sample.getContext('2d')
    if (!context) throw new Error('Screenshot decoder unavailable')
    context.drawImage(image, 0, 0)
    const pixels = context.getImageData(0, 0, sample.width, sample.height).data
    const distinct = new Set<string>()
    for (let index = 0; index < pixels.length; index += 16) {
      distinct.add(`${pixels[index]},${pixels[index + 1]},${pixels[index + 2]}`)
    }
    return distinct.size
  }, screenshot.toString('base64'))
  expect(colors).toBeGreaterThan(10)
}
