import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor } from '../tests/support/duel'
import type { GameServerMessage } from '../../web/src/game/v19/schema'

test.beforeAll(() => {
  expect(process.env.E2E_CPU_ITEM_FIXTURE).toBe('arena-v3-cpu-wall-pass')
})

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>

test('native CPU collects Wall Pass with all items enabled and renders its public gear', async ({ page }) => {
  const game = observeGame(page)
  await page.goto('/')
  await page.getByRole('button', { name: 'Play solo against CPU', exact: true }).click()
  await expect.poll(() => game.frames.findLast((frame) =>
    frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot'))
    .toBeTruthy()
  const room = game.frames.findLast((frame) =>
    frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot')!.room
  expect(room.availableItems).toHaveLength(22)
  expect(room.settings.enabledItems).toEqual(room.availableItems)
  expect(room.settings.environment).toEqual({
    stormCloud: true,
    lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
  })

  const revision = room.settingsRevision
  await page.getByRole('checkbox', { name: 'Storm cloud', exact: true }).uncheck()
  await page.getByRole('checkbox', { name: 'Monster lairs', exact: true }).uncheck()
  await page.getByLabel('Bomb fuse').selectOption('300')
  await page.getByRole('button', { name: 'Apply rules', exact: true }).click()
  await expect.poll(() => game.frames.findLast((frame) =>
    frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot')
    ?.room.settingsRevision).toBeGreaterThan(revision)
  const updatedRoom = () => game.frames.findLast((frame) =>
    frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot')
    ?.room
  await expect.poll(() => updatedRoom()?.settings.enabledItems).toEqual(room.availableItems)
  const environment = {
    stormCloud: false,
    lairs: { enabled: false, spawnSeconds: 10, maxLivingMobs: 1 },
  }
  await expect.poll(() => updatedRoom()?.settings.environment).toEqual(environment)
  expect(updatedRoom()?.configuration.environment).toEqual(environment)

  await page.getByRole('button', { name: 'Ready', exact: true }).click()
  await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
  const snapshots = (): Snapshot[] => game.frames.filter(
    (frame): frame is Snapshot => frame.type === 'snapshot')
  await expect.poll(() => snapshots().flatMap(({ events }) => events).some(({ kind }) =>
    kind.type === 'pickupCollected' && kind.seat === 'two' && kind.kind === 'wallPass'),
  { timeout: 90_000 }).toBe(true)
  await expect.poll(() => visibleActor(latestSnapshot(game), 'two')?.inventory.equipment)
    .toBe('wallPass')
  const cpuPlayer = page.getByRole('list', { name: 'Players' }).getByRole('listitem')
    .filter({ has: page.getByText('CPU opponent', { exact: true }) })
  await expect(cpuPlayer).toHaveCount(1)
  const gear = cpuPlayer.locator('dl > div')
    .filter({ has: page.getByRole('term').filter({ hasText: /^Gear$/ }) })
  await expect(gear.getByRole('term')).toHaveText('Gear')
  await expect(gear.getByRole('definition')).toHaveText('Wall pass')
})
