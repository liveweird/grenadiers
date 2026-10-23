import { expect, type Page } from '@playwright/test'
import { visibleActor, latestSnapshot, type ObservedGame } from './duel'
import type { Seat } from '../../../web/src/game/v10/schema'

type Cell = { column: number; row: number }
const CENTER = 512
const CELL = 1024
const RELEASE_LEAD = 640
const BASE_SPEED = 3_072

function actor(game: ObservedGame, seat: Seat) {
  const value = visibleActor(latestSnapshot(game), seat)
  if (!value?.alive) throw new Error(`Fixture route requires a living ${seat} player`)
  return value
}

export async function moveSeatTo(
  page: Page,
  game: ObservedGame,
  seat: Seat,
  cell: Cell,
  axisOrder: readonly ('x' | 'y')[] = ['x', 'y'],
) {
  const target = { x: cell.column * CELL + CENTER, y: cell.row * CELL + CENTER }
  for (let attempt = 0; attempt < 20; attempt += 1) {
    const position = actor(game, seat).position
    const differences = { x: target.x - position.x, y: target.y - position.y }
    if (Math.abs(differences.x) <= 128 && Math.abs(differences.y) <= 128) break
    const axis = axisOrder.find((candidate) => Math.abs(differences[candidate]) > 128)!
    const delta = differences[axis]
    const keys = axis === 'x' ? ['ArrowLeft', 'ArrowRight'] : ['ArrowUp', 'ArrowDown']
    const directions = axis === 'x' ? ['west', 'east'] : ['north', 'south']
    const key = keys[Number(delta >= 0)]
    const direction = directions[Number(delta >= 0)]
    const startRevision = latestSnapshot(game)!.room.revision
    const startPosition = position[axis]
    const commandCount = game.commands.length
    if (Math.abs(delta) <= RELEASE_LEAD) {
      // Keep down, delay, and up in one Playwright operation: polling between
      // them can lag behind a loaded browser and turn a fine correction into a
      // full TTL movement. Cap the pulse below the final-position window.
      const pulseMs = Math.max(34, Math.min(60, Math.round(Math.abs(delta) * 1_000 / BASE_SPEED)))
      await page.keyboard.press(key, { delay: pulseMs })
    } else {
      await page.keyboard.down(key)
      try {
        await expect.poll(() => game.commands.slice(commandCount)
          .some((command) => command.type === 'movement' && command.direction === direction))
          .toBe(true)
        const sign = Math.sign(delta)
        await expect.poll(() => {
          const remaining = target[axis] - actor(game, seat).position[axis]
          return Math.abs(remaining) <= RELEASE_LEAD || Math.sign(remaining) !== sign
        }, { intervals: [15] }).toBe(true)
      } finally {
        await page.keyboard.up(key)
      }
    }
    await expect.poll(() => {
      const commands = game.commands.slice(commandCount)
      const movement = commands.findIndex((command) => command.type === 'movement'
        && command.direction === direction)
      return movement >= 0 && commands.slice(movement + 1)
        .some((command) => command.type === 'movement' && command.direction === null)
    }, { intervals: [15] })
      .toBe(true)
    // Do not mistake snapshots queued before the input for proof of key-up. The
    // accepted direction must move the actor before stationary frames can settle
    // the pulse; this also prevents an opposite correction from being queued.
    await expect.poll(() => actor(game, seat).position[axis], { intervals: [15], timeout: 1_000 })
      .not.toBe(startPosition)
    const movedRevision = latestSnapshot(game)!.room.revision
    await expect.poll(() => {
      const frames = game.frames.filter((frame) => frame.type === 'snapshot').slice(-2)
      if (frames.length !== 2 || frames[0].room.revision <= Math.max(startRevision, movedRevision)) return false
      const positions = frames.map((frame) => visibleActor(frame, seat)!.position)
      return positions.every((value) => value.x === positions[0].x && value.y === positions[0].y)
    }, { intervals: [15] }).toBe(true)
  }
  expect(Math.abs(target.x - actor(game, seat).position.x)).toBeLessThanOrEqual(128)
  expect(Math.abs(target.y - actor(game, seat).position.y)).toBeLessThanOrEqual(128)
}

export async function moveTo(
  page: Page,
  game: ObservedGame,
  cell: Cell,
  axisOrder: readonly ('x' | 'y')[] = ['x', 'y'],
) { return moveSeatTo(page, game, 'one', cell, axisOrder) }

