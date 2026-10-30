import { expect, type Page } from '@playwright/test'
import type { GameServerMessage } from '../../../web/src/game/v17/schema'
import { latestSnapshot, visibleActor, type ObservedGame } from './duel'

export type EffectPickupKind = 'goldenFlame' | 'shortFuse' | 'shortFlame'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>

export function latestPublicRoom(game: ObservedGame) {
  return game.frames.filter((frame) => frame.type === 'admitted'
    || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room
}

export function effectTicks(game: ObservedGame, kind: EffectPickupKind): number {
  return visibleActor(latestSnapshot(game), 'one')?.effects
    .find((effect) => effect.kind === kind)?.remainingTicks ?? 0
}

export function ownCollectionTick(game: ObservedGame, kind: EffectPickupKind): number {
  const event = game.frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    .find(({ kind: eventKind }) => eventKind.type === 'pickupCollected'
      && eventKind.seat === 'one' && eventKind.kind === kind)
  expect(event, 'The real pickup event establishes the effect deadline').toBeDefined()
  return event!.tick
}

export function expectExact600TickExpiry(
  game: ObservedGame,
  collectedAt: number,
  kind: EffectPickupKind,
  label: string,
): void {
  const expiresAt = collectedAt + 600
  const frames = game.frames.filter((frame): frame is Snapshot => frame.type === 'snapshot'
    && frame.state.physicalTick >= collectedAt)
  expect(frames.length).toBeGreaterThan(1)
  for (const frame of frames) {
    const effect = visibleActor(frame, 'one')?.effects.find((value) => value.kind === kind)
    expect(effect?.remainingTicks ?? 0, `${label} at physical tick ${frame.state.physicalTick}`)
      .toBe(Math.max(0, expiresAt - frame.state.physicalTick))
  }
  const firstExpired = frames.find((frame) => frame.state.physicalTick >= expiresAt)
  expect(firstExpired, 'A publication must cross the exact effect deadline').toBeDefined()
  expect(frames.some((frame) => frame.state.physicalTick < expiresAt)).toBe(true)
}

export async function plantBombWithNativeSpace(page: Page, game: ObservedGame): Promise<number> {
  const previous = latestSnapshot(game)!.state.bombs.map((bomb) => bomb.id)
  await page.keyboard.press('Space')
  const newBomb = () => latestSnapshot(game)?.state.bombs.find((bomb) => !previous.includes(bomb.id))
  await expect.poll(() => Boolean(newBomb()), { intervals: [15] }).toBe(true)
  const bomb = newBomb()
  if (!bomb) throw new Error('Native Space input must place a new bomb')
  return bomb.id
}
