import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test, type Page } from '@playwright/test'
import { captureArenaTiles } from '../tests/support/compositor'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage } from '../../web/src/game/v20/schema'

test.beforeAll(() => expect(process.env.E2E_PUNCH_FIXTURE).toBe('arena-v3-punch'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type Admission = Extract<GameServerMessage, { type: 'admitted' }>
type StatefulAdmission = Admission & { state: NonNullable<Admission['state']> }
type Bomb = Snapshot['state']['bombs'][number]
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const actor = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')!
const eventsAfter = (game: ObservedGame, revision: number) => snapshots(game)
  .filter(({ room }) => room.revision > revision).flatMap(({ events }) => events)
const placedAfter = (game: ObservedGame, revision: number) => eventsAfter(game, revision)
  .flatMap(({ kind }) => kind.type === 'bombPlaced' ? [kind.bomb] : [])
const detonatedAfter = (game: ObservedGame, revision: number) => eventsAfter(game, revision)
  .flatMap(({ tick, kind }) => kind.type === 'bombDetonated' ? [{ tick, bomb: kind.bomb }] : [])
const punchPrivateFacts = /lastSuccessfulPunchTick|hasBeenPunched/i
const revisionFrames = (game: ObservedGame, after: number, through: number): Snapshot[] =>
  snapshots(game).filter(({ room }) => room.revision > after && room.revision <= through)

async function waitClear(game: ObservedGame): Promise<void> {
  await expect.poll(() => {
    const state = latestSnapshot(game)!.state
    return state.bombs.length + state.flames.length
  }).toBe(0)
}

async function revealAndCollect(page: Page, game: ObservedGame): Promise<void> {
  await moveSeatTo(page, game, 'one', { column: 2, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await expect.poll(() => actor(game).inventory.equipment).toBe('punch')
}

async function clearPunchRoute(page: Page, game: ObservedGame): Promise<void> {
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 1, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 4, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 2 })
  await waitClear(game)
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 4, row: 1 })
}

async function plantDemoBomb(page: Page, game: ObservedGame): Promise<{
  beforePlant: number, demo: Bomb, actionId: number,
}> {
  const beforePlant = latestSnapshot(game)!.room.revision
  const commandFloor = game.commands.length
  await page.keyboard.press('Space')
  await expect.poll(() => placedAfter(game, beforePlant)).toHaveLength(1)
  const demo = placedAfter(game, beforePlant)[0]!
  const plant = game.commands.slice(commandFloor)
    .find((command) => command.type === 'manualAction' && command.action === 'bomb')!
  expect(demo).toMatchObject({
    cell: { column: 4, row: 1 }, owner: actor(game).id,
    blast: 'standard', trigger: { type: 'timed', remainingTicks: 300, paused: false },
  })
  return { beforePlant, demo, actionId: plant.actionId! }
}

async function punchBombByContact(
  page: Page, games: ObservedGame[], demo: Bomb,
): Promise<{ punchFloors: number[], movedRevisions: number[] }> {
  await moveSeatTo(page, games[0], 'one', { column: 5, row: 1 })
  const actorFacts = games.map((game) => {
    const value = actor(game)
    return { position: value.position, lastTeleportTick: value.lastTeleportTick }
  })
  const punchFloors = games.map((game) => latestSnapshot(game)!.room.revision)
  const commandFloor = games[0].commands.length
  await page.keyboard.press('ArrowLeft', { delay: 40 })
  await expect.poll(() => games[0].commands.slice(commandFloor)).toEqual([
    { type: 'movement', direction: 'west' }, { type: 'movement', direction: null },
  ])
  await Promise.all(games.map((game, index) => expect.poll(() => {
    const snapshot = latestSnapshot(game)!
    if (snapshot.room.revision <= punchFloors[index]!) return undefined
    return snapshot.state.bombs.find(({ id }) => id === demo.id)?.cell
  }).toEqual({ column: 1, row: 1 })))

  const movedRevisions: number[] = []
  for (const [index, game] of games.entries()) {
    const moved = snapshots(game).find(({ room, state }) => room.revision > punchFloors[index]!
      && state.bombs.some(({ id, cell }) => id === demo.id && cell.column === 1 && cell.row === 1))!
    movedRevisions[index] = moved.room.revision
    expect(moved.state.bombs.find(({ id }) => id === demo.id)).toMatchObject({
      id: demo.id, cell: { column: 1, row: 1 }, owner: demo.owner,
      range: demo.range, placedAt: demo.placedAt, blast: demo.blast,
      trigger: expect.objectContaining({ type: 'timed', paused: false }),
    })
    const rawPunchFrames = revisionFrames(game, punchFloors[index]!, moved.room.revision)
    expect(rawPunchFrames.flatMap(({ events }) => events)
      .filter(({ kind }) => ['bombPlaced', 'bombDetonated'].includes(kind.type))).toEqual([])
    expect(visibleActor(moved, 'one')).toMatchObject(actorFacts[index]!)
    expect(JSON.stringify(rawPunchFrames)).not.toMatch(punchPrivateFacts)
  }
  return { punchFloors, movedRevisions }
}

async function expectNaturalDetonation(
  games: ObservedGame[], demo: Bomb, beforePlant: number,
  punchFloors: number[], movedRevisions: number[],
): Promise<void> {
  await Promise.all(games.map((game) => expect.poll(() => detonatedAfter(game, beforePlant)
    .find(({ bomb }) => bomb.id === demo.id)).toBeTruthy()))
  for (const [index, game] of games.entries()) {
    const detonation = detonatedAfter(game, beforePlant).find(({ bomb }) => bomb.id === demo.id)!
    const naturalFrames = revisionFrames(game, movedRevisions[index]!, latestSnapshot(game)!.room.revision)
    expect(naturalFrames.flatMap(({ events }) => events).some(({ kind }) =>
      kind.type === 'bombDetonated' && kind.bomb.id === demo.id)).toBe(true)
    expect(JSON.stringify(naturalFrames)).not.toMatch(punchPrivateFacts)
    expect(detonation.tick).toBe(demo.placedAt + 300)
    expect(detonation.bomb).toMatchObject({ id: demo.id, cell: { column: 1, row: 1 } })
    expect(latestSnapshot(game)!.room.revision).toBeGreaterThan(punchFloors[index]!)
  }
}

async function expectResumedContactState(
  page: Page, game: ObservedGame, demo: Bomb, actionId: number,
): Promise<void> {
  const beforeReconnect = latestSnapshot(game)!
  const expectedBomb = beforeReconnect.state.bombs.find(({ id }) => id === demo.id)!
  if (expectedBomb.trigger.type !== 'timed') throw new Error('Punch resume bomb is not timed')
  const fuseDeadline = demo.placedAt + 300
  expect(beforeReconnect.state.physicalTick + expectedBomb.trigger.remainingTicks).toBe(fuseDeadline)
  const baseline = (await admissionBaseline(page))!
  await disconnectTransport(page)
  await expect.poll(async () => (await admissionBaseline(page))?.index).toBeGreaterThan(baseline.index)
  await expect.poll(() => game.frames.some((frame) => frame.type === 'admitted'
    && frame.room.roomId === beforeReconnect.room.roomId && frame.youSeat === 'one'
    && frame.room.revision > beforeReconnect.room.revision && frame.state !== null)).toBe(true)
  const resumed = game.frames.findLast((frame): frame is StatefulAdmission => frame.type === 'admitted'
    && frame.room.roomId === beforeReconnect.room.roomId && frame.youSeat === 'one'
    && frame.room.revision > beforeReconnect.room.revision && frame.state !== null)!
  expect(resumed.lastActionId).toBe(actionId)
  const resumedBomb = resumed.state.bombs.find(({ id }) => id === demo.id)!
  expect(resumedBomb).toMatchObject({
    id: expectedBomb.id, cell: expectedBomb.cell, owner: expectedBomb.owner,
    range: expectedBomb.range, placedAt: expectedBomb.placedAt, blast: expectedBomb.blast,
  })
  if (resumedBomb.trigger.type !== 'timed') throw new Error('Resumed Punch bomb is not timed')
  expect(resumedBomb.trigger.paused).toBe(expectedBomb.trigger.paused)
  expect(resumedBomb.trigger.remainingTicks).toBeGreaterThan(0)
  expect(resumedBomb.trigger.remainingTicks).toBeLessThanOrEqual(expectedBomb.trigger.remainingTicks)
  expect(resumed.state.physicalTick + resumedBomb.trigger.remainingTicks).toBe(fuseDeadline)
  expect(JSON.stringify(resumed.state)).not.toMatch(punchPrivateFacts)
  expect((await admissionBaseline(page))?.equipment)
    .toContainEqual({ seat: 'one', equipment: 'punch' })

  const commandFloor = game.commands.length
  const revision = latestSnapshot(game)!.room.revision
  await page.getByRole('region', { name: 'Grenadiers courtyard' }).click()
  await page.keyboard.press('KeyE')
  await expect.poll(() => latestSnapshot(game)!.room.revision).toBeGreaterThan(revision)
  expect(game.commands).toHaveLength(commandFloor)
}

test('Punch wraps one retained bomb on contact and preserves its fuse', async ({ browser }, info) => {
  const contexts = await createResponsiveBrowserContexts(browser)
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two].map(installTransportProbe))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one.goto('/'), two.goto('/')])
    await expect(one.getByText(/With Punch equipped, walk into a bomb/)).toBeVisible()
    await one.getByRole('button', { name: 'Keyboard controls', exact: true }).click()
    await expect(one.getByRole('dialog', { name: 'Keyboard controls' })).not.toContainText('Equipment')
    await one.getByRole('button', { name: 'Done', exact: true }).click()

    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await applyItemSettings(one, games[0], [{ id: 'punch', label: 'Punch' }],
      () => one.getByLabel('Bomb fuse').selectOption('300'))
    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await Promise.all([
      one.getByRole('button', { name: 'Ready', exact: true }).click(),
      two.getByRole('button', { name: 'Gotowy', exact: true }).click(),
    ])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
      .toBe('running')))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await revealAndCollect(one, games[0])
    await clearPunchRoute(one, games[0])
    await expect(one.getByRole('region', { name: 'Your items' })).toContainText('GearPunch')
    await expect(one.getByText(/Walk into a stationary timed or remote bomb/)).toBeVisible()
    await expect(two.getByText(/Wejdź na nieruchomą bombę czasową lub zdalną/)).toBeVisible()

    const first = await plantDemoBomb(one, games[0])
    const firstPunch = await punchBombByContact(one, games, first.demo)
    const cells = [{ column: 1, row: 1 }, { column: 4, row: 1 }]
    const captures = await Promise.all([
      captureArenaTiles(one, info.outputPath('punch-contact-desktop-en.png'), cells),
      captureArenaTiles(two, info.outputPath('punch-contact-polish-320-reduced.png'), cells),
    ])
    expect(captures.every(({ screenshot }) => screenshot.length > 0)).toBe(true)
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
    await expectNaturalDetonation(games, first.demo, first.beforePlant,
      firstPunch.punchFloors, firstPunch.movedRevisions)

    await waitClear(games[0])
    await moveSeatTo(one, games[0], 'one', { column: 4, row: 1 })
    const resumed = await plantDemoBomb(one, games[0])
    const resumedPunch = await punchBombByContact(one, games, resumed.demo)
    await expectResumedContactState(one, games[0], resumed.demo, resumed.actionId)
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
    await expectNaturalDetonation(games, resumed.demo, resumed.beforePlant,
      resumedPunch.punchFloors, resumedPunch.movedRevisions)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
