import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import type { GameServerMessage, Seat } from '../../web/src/game/v18/schema'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

function firstCpuCollection(game: ObservedGame): { frame: Snapshot; seat: Seat } | undefined {
  for (const frame of snapshots(game)) {
    for (const { kind } of frame.events) {
      if (kind.type === 'pickupCollected' && kind.kind === 'goldenFlame' && kind.seat !== 'one') {
        return { frame, seat: kind.seat }
      }
    }
  }
  return undefined
}

test.beforeAll(() => expect(process.env.E2E_CPU_PICKUP_FIXTURE).toBe('arena-v3-cpu-pickup'))

test('CPUs collect exposed Golden Flame and keep moving after the pickup handoff', async ({ browser }) => {
  const context = await browser.newContext()
  try {
    const page = await context.newPage()
    const game = observeGame(page)
    await page.goto('/')
    await page.getByRole('combobox', { name: 'Players', exact: true }).selectOption('4')
    for (const index of [2, 3, 4]) {
      await page.getByRole('combobox', { name: `Player ${index} controller`, exact: true }).selectOption('cpu')
    }
    await page.getByRole('combobox', { name: 'CPU difficulty', exact: true }).last().selectOption('standard')
    await page.getByRole('button', { name: 'Create custom match', exact: true }).click()
    await expect.poll(() => latestRoom(game)).toBeTruthy()
    const revision = latestRoom(game)!.settingsRevision
    await page.getByRole('button', { name: 'All off', exact: true }).click()
    for (const label of ['Golden flame', 'Storm cloud', 'Monster lairs']) {
      await page.getByRole('checkbox', { name: label, exact: true }).check()
    }
    await page.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => latestRoom(game)?.settingsRevision).toBeGreaterThan(revision)
    expect(latestRoom(game)?.settings.enabledItems).toEqual(['goldenFlame'])
    expect(latestRoom(game)?.settings.environment).toEqual({
      stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
    })
    await page.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
    await page.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await page.keyboard.press('Space')
    await expect.poll(() => visibleActor(latestSnapshot(game), 'one')?.alive).toBe(false)
    await expect.poll(() => firstCpuCollection(game), { timeout: 20_000 }).toBeTruthy()
    const collected = firstCpuCollection(game)!
    const origin = visibleActor(collected.frame, collected.seat)!.position
    await expect.poll(() => snapshots(game).some((frame) => {
      if (frame.room.roundId !== collected.frame.room.roundId
        || frame.state.physicalTick <= collected.frame.state.physicalTick + 1) return false
      const actor = visibleActor(frame, collected.seat)
      return actor?.alive && Math.abs(actor.position.x - origin.x)
        + Math.abs(actor.position.y - origin.y) > 128
    }), { timeout: 10_000 }).toBe(true)
  } finally { await context.close() }
})
