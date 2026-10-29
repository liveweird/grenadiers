import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test, type Page } from '@playwright/test'
import { captureArenaTiles } from '../tests/support/compositor'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage } from '../../web/src/game/v16/schema'

test.beforeAll(() => expect(process.env.E2E_PUNCH_FIXTURE).toBe('arena-v3-punch'))
type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type Admission = Extract<GameServerMessage, { type: 'admitted' }>
type StatefulAdmission = Admission & { state: NonNullable<Admission['state']> }
type Bomb = Snapshot['state']['bombs'][number]
type Command = ObservedGame['commands'][number]
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

async function expectLiveCaptureScene(
  games: ObservedGame[], bombId: number, position: { x: number, y: number },
): Promise<void> {
  await Promise.all(games.map((game) => expect.poll(() => {
    const snapshot = latestSnapshot(game)!
    const bomb = snapshot.state.bombs.find(({ id }) => id === bombId)
    return {
      bomb: bomb?.cell,
      flames: snapshot.state.flames.length,
      one: { alive: visibleActor(snapshot, 'one')?.alive,
        position: visibleActor(snapshot, 'one')?.position },
      twoAlive: visibleActor(snapshot, 'two')?.alive,
    }
  }).toEqual({
    bomb: { column: 1, row: 1 }, flames: 0,
    one: { alive: true, position }, twoAlive: true,
  })))
}

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
  for (const cell of [{ column: 3, row: 2 }, { column: 4, row: 1 }]) {
    expect(latestSnapshot(game)!.state.arena.blocks).not.toContainEqual(cell)
  }
  await moveSeatTo(page, game, 'one', { column: 1, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 4, row: 1 })
  await page.keyboard.press('Space')
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 2 })
  await waitClear(game)
  expect(latestSnapshot(game)!.state.arena.blocks).not.toContainEqual({ column: 5, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 3, row: 1 })
  await moveSeatTo(page, game, 'one', { column: 4, row: 1 })
}

async function plantDemoBomb(page: Page, game: ObservedGame): Promise<{
  beforePlant: number, demo: Bomb, plant: Command,
}> {
  const beforePlant = latestSnapshot(game)!.room.revision
  const plantCommands = game.commands.length
  await page.keyboard.press('Space')
  await expect.poll(() => placedAfter(game, beforePlant)).toHaveLength(1)
  const demo = placedAfter(game, beforePlant)[0]!
  const plant = game.commands.slice(plantCommands)
    .find((command) => command.type === 'manualAction' && command.action === 'bomb')!
  expect(demo).toMatchObject({
    cell: { column: 4, row: 1 }, owner: actor(game).id,
    blast: 'standard', trigger: { type: 'timed', remainingTicks: 300, paused: false },
  })
  return { beforePlant, demo, plant }
}

async function punchBombWest(
  page: Page, games: ObservedGame[], demo: Bomb, plant: Command,
): Promise<{ punch: Command, punchFloors: number[], movedRevisions: number[] }> {
  await moveSeatTo(page, games[0], 'one', { column: 5, row: 1 })
  const faceFloors = games.map((game) => latestSnapshot(game)!.room.revision)
  const faceCommands = games[0].commands.length
  await page.keyboard.press('ArrowLeft', { delay: 40 })
  await expect.poll(() => {
    const commands = games[0].commands.slice(faceCommands)
    const west = commands.findIndex((command) => command.type === 'movement'
      && command.direction === 'west')
    return west >= 0 && commands.slice(west + 1).some((command) => command.type === 'movement'
      && command.direction === null)
  }, { intervals: [15] }).toBe(true)
  await Promise.all(games.map((game, index) => expect.poll(() => {
    const frames = snapshots(game).filter(({ room }) => room.revision > faceFloors[index]!).slice(-2)
    if (frames.length !== 2 || frames[0]!.state.physicalTick >= frames[1]!.state.physicalTick) {
      return false
    }
    const actors = frames.map((snapshot) => visibleActor(snapshot, 'one'))
    return actors.every((value) => value !== undefined && value.heading === 'west'
      && value.position.x === actors[0]!.position.x
      && value.position.y === actors[0]!.position.y)
      && frames.every((snapshot) => snapshot.state.bombs.some(({ id, cell }) => id === demo.id
        && cell.column === 4 && cell.row === 1))
  }, { intervals: [15] }).toBe(true)))
  const actorFacts = games.map((game) => {
    const value = visibleActor(latestSnapshot(game), 'one')!
    return { position: value.position, heading: value.heading, lastTeleportTick: value.lastTeleportTick }
  })
  const punchFloors = games.map((game) => latestSnapshot(game)!.room.revision)
  const punchCommands = games[0].commands.length
  await page.keyboard.press('KeyF')
  await expect.poll(() => games[0].commands.slice(punchCommands)
    .find((command) => command.type === 'manualAction' && command.action === 'equipment'))
    .toMatchObject({ actionId: plant.actionId! + 1 })
  const punch = games[0].commands.slice(punchCommands)
    .find((command) => command.type === 'manualAction' && command.action === 'equipment')!
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
    const bomb = moved.state.bombs.find(({ id }) => id === demo.id)!
    expect(bomb).toMatchObject({
      id: demo.id, cell: { column: 1, row: 1 }, owner: demo.owner,
      range: demo.range, placedAt: demo.placedAt, blast: demo.blast,
      trigger: expect.objectContaining({ type: 'timed', paused: false }),
    })
    const rawPunchFrames = revisionFrames(game, punchFloors[index]!, moved.room.revision)
    expect(rawPunchFrames.length).toBeGreaterThan(0)
    expect(rawPunchFrames.flatMap(({ events }) => events)
      .filter(({ kind }) => ['bombPlaced', 'bombDetonated'].includes(kind.type)))
      .toEqual([])
    expect(visibleActor(moved, 'one')).toMatchObject(actorFacts[index]!)
    expect(moved.state.bombs.some(({ cell }) => cell.column === 4 && cell.row === 1)).toBe(false)
    expect(JSON.stringify(rawPunchFrames)).not.toMatch(punchPrivateFacts)
  }
  return { punch, punchFloors, movedRevisions }
}

async function expectNaturalDetonation(
  games: ObservedGame[], demo: Bomb, beforePlant: number,
  punchFloors: number[], movedRevisions: number[],
): Promise<void> {
  await Promise.all(games.map((game) => expect.poll(() => detonatedAfter(game, beforePlant)
    .find(({ bomb }) => bomb.id === demo.id)).toBeTruthy()))
  for (const [index, game] of games.entries()) {
    const detonation = detonatedAfter(game, beforePlant).find(({ bomb }) => bomb.id === demo.id)!
    const detonationFrame = snapshots(game).find(({ room, events }) =>
      room.revision > punchFloors[index]! && events.some(({ kind }) =>
        kind.type === 'bombDetonated' && kind.bomb.id === demo.id))!
    await expect.poll(() => latestSnapshot(game)!.room.revision)
      .toBeGreaterThan(detonationFrame.room.revision)
    const naturalFrames = revisionFrames(game, movedRevisions[index]!,
      latestSnapshot(game)!.room.revision)
    expect(naturalFrames.flatMap(({ events }) => events).some(({ kind }) =>
      kind.type === 'bombDetonated' && kind.bomb.id === demo.id)).toBe(true)
    expect(JSON.stringify(naturalFrames)).not.toMatch(punchPrivateFacts)
    expect(detonation.tick).toBe(demo.placedAt + 300)
    expect(detonation.bomb).toMatchObject({
      id: demo.id, cell: { column: 1, row: 1 }, owner: demo.owner,
      range: demo.range, placedAt: demo.placedAt, blast: demo.blast,
    })
    for (const seat of ['one', 'two'] as const) {
      expect(visibleActor(latestSnapshot(game), seat)?.alive).toBe(true)
    }
  }
}

async function resumePunchedBombAndSendEquipment(
  page: Page, games: ObservedGame[], demo: Bomb, punch: Command,
): Promise<{ actionId: number, commandFloor: number }> {
  const game = games[0]
  const beforeReconnect = latestSnapshot(game)!
  const expectedBomb = beforeReconnect.state.bombs.find(({ id }) => id === demo.id)!
  if (expectedBomb.trigger.type !== 'timed') throw new Error('Punch resume bomb is not Timed')
  const fuseDeadline = demo.placedAt + 300
  expect(beforeReconnect.state.physicalTick + expectedBomb.trigger.remainingTicks).toBe(fuseDeadline)
  const expectedActor = visibleActor(beforeReconnect, 'one')!
  const reconnectCommandFloor = game.commands.length
  const reconnectSocketFloor = game.sockets
  const baseline = (await admissionBaseline(page))!
  await disconnectTransport(page)
  await expect.poll(() => game.sockets).toBeGreaterThan(reconnectSocketFloor)
  await expect.poll(async () => (await admissionBaseline(page))?.index).toBeGreaterThan(baseline.index)
  // The observer retains a bounded buffer; revision fences survive its rotation.
  await expect.poll(() => game.frames.some((frame) =>
    frame.type === 'admitted' && frame.room.roomId === beforeReconnect.room.roomId
    && frame.room.roundId === beforeReconnect.room.roundId && frame.youSeat === 'one'
    && frame.room.revision > beforeReconnect.room.revision && frame.state !== null)).toBe(true)
  const resumed = game.frames
    .findLast((frame): frame is StatefulAdmission => frame.type === 'admitted'
      && frame.room.roomId === beforeReconnect.room.roomId
      && frame.room.roundId === beforeReconnect.room.roundId && frame.youSeat === 'one'
      && frame.room.revision > beforeReconnect.room.revision && frame.state !== null)!
  expect(resumed.lastActionId).toBe(punch.actionId)
  const resumedBomb = resumed.state.bombs.find(({ id }) => id === demo.id)!
  expect(resumedBomb).toMatchObject({
    id: expectedBomb.id, cell: expectedBomb.cell, owner: expectedBomb.owner,
    range: expectedBomb.range, placedAt: expectedBomb.placedAt, blast: expectedBomb.blast,
  })
  if (resumedBomb.trigger.type !== 'timed') throw new Error('Resumed Punch bomb is not Timed')
  expect(resumedBomb.trigger.paused).toBe(expectedBomb.trigger.paused)
  expect(resumedBomb.trigger.remainingTicks).toBeGreaterThan(0)
  expect(resumedBomb.trigger.remainingTicks).toBeLessThanOrEqual(expectedBomb.trigger.remainingTicks)
  expect(resumed.state.physicalTick + resumedBomb.trigger.remainingTicks).toBe(fuseDeadline)
  expect(resumed.state.actors.find((value) => value.type === 'visible' && value.seat === 'one'))
    .toMatchObject({
      seat: 'one', alive: expectedActor.alive, position: expectedActor.position,
      heading: expectedActor.heading, inventory: expectedActor.inventory,
    })
  expect(resumed.state.actors.find((value) => value.type === 'visible' && value.seat === 'two'))
    .toMatchObject({ seat: 'two', alive: true })
  expect(JSON.stringify(resumed.state)).not.toMatch(punchPrivateFacts)
  expect('events' in resumed).toBe(false)
  expect(game.commands.slice(reconnectCommandFloor)
    .filter((command) => command.type === 'manualAction' && command.action === 'equipment'))
    .toEqual([])
  const resumedBaseline = (await admissionBaseline(page))!
  expect(resumedBaseline).toMatchObject({
    roomId: resumed.room.roomId, seat: 'one', actionFloor: punch.actionId,
    hasState: true, stableCapability: true,
  })
  expect(resumedBaseline.equipment).toContainEqual({ seat: 'one', equipment: 'punch' })
  expect(resumedBaseline.bombIds).toContain(demo.id)

  const commandFloor = game.commands.length
  const postResumeRevision = latestSnapshot(game)!.room.revision
  await page.getByRole('region', { name: 'Grenadiers courtyard' }).click()
  await page.keyboard.press('KeyF')
  const actionId = punch.actionId! + 1
  await expect.poll(() => game.commands.slice(commandFloor)
    .filter((command) => command.type === 'manualAction' && command.action === 'equipment'))
    .toEqual([{ type: 'manualAction', action: 'equipment', actionId }])
  await expect.poll(() => latestSnapshot(game)!.room.revision).toBeGreaterThan(postResumeRevision)
  expect(latestSnapshot(game)!.state.bombs.find(({ id }) => id === demo.id)?.cell)
    .toEqual({ column: 1, row: 1 })
  return { actionId, commandFloor }
}

async function confirmResumedEquipmentAccepted(
  page: Page, game: ObservedGame, proof: { actionId: number, commandFloor: number },
): Promise<void> {
  const baseline = (await admissionBaseline(page))!
  await disconnectTransport(page)
  await expect.poll(async () => (await admissionBaseline(page))?.index).toBeGreaterThan(baseline.index)
  expect((await admissionBaseline(page))?.actionFloor).toBe(proof.actionId)
  expect(game.commands.slice(proof.commandFloor)
    .filter((command) => command.type === 'manualAction' && command.action === 'equipment'))
    .toEqual([{ type: 'manualAction', action: 'equipment', actionId: proof.actionId }])
}

test('Punch wraps one retained bomb three cells without extra cues and preserves its fuse',
  async ({ browser }, info) => {
    const contexts = await createResponsiveBrowserContexts(browser)
    try {
      const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
      await Promise.all([one, two].map(installTransportProbe))
      const games = [observeGame(one), observeGame(two)]
      await Promise.all([one.goto('/'), two.goto('/')])
      await one.getByRole('button', { name: 'Keyboard controls', exact: true }).click()
      await one.getByRole('button', { name: 'Change Equipment', exact: true }).click()
      await one.keyboard.press('KeyF')
      await expect(one.getByRole('dialog', { name: 'Keyboard controls' })).toContainText('EquipmentF')
      await one.getByRole('button', { name: 'Done', exact: true }).click()
      await expect(one.getByText(/Use Punch equipment with F/)).toBeVisible()

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
      await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()
      await revealAndCollect(one, games[0])
      await clearPunchRoute(one, games[0])
      await Promise.all(games.map((game) => expect.poll(() => actor(game).inventory.equipment)
        .toBe('punch')))
      await expect(one.getByRole('region', { name: 'Your items' })).toContainText('GearPunch')
      await expect(two.getByLabel('Ulepszenia: Gracz pierwszy')).toContainText('WyposażenieUderzenie')
      await one.getByRole('button', { name: 'Keyboard controls', exact: true }).focus()
      await expect(one.getByText(/press F to punch/)).toBeVisible()
      await expect(one.getByText(/fixed 0.2 second cooldown/)).toBeVisible()
      await expect(two.getByText(/stałe 0,2 sekundy odnowienia/)).toBeVisible()
      await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()

      const { beforePlant, demo, plant } = await plantDemoBomb(one, games[0])
      const { punchFloors, movedRevisions } = await punchBombWest(one, games, demo, plant)

      await moveSeatTo(one, games[0], 'one', { column: 4, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
      const capturePosition = actor(games[0]).position
      await expectLiveCaptureScene(games, demo.id, capturePosition)
      const cells = [{ column: 1, row: 1 }, { column: 4, row: 1 }]
      const captures = await Promise.all([
        captureArenaTiles(one, info.outputPath('punch-wrapped-desktop-en.png'), cells),
        captureArenaTiles(two, info.outputPath('punch-wrapped-polish-320-reduced.png'), cells),
      ])
      await expectLiveCaptureScene(games, demo.id, capturePosition)
      expect(captures[0].pixels[0]).not.toEqual(captures[0].pixels[1])
      expect(captures[1].pixels[0]).not.toEqual(captures[1].pixels[1])
      expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
      expect(captures[0].screenshot.length).toBeGreaterThan(0)
      expect(captures[1].screenshot.length).toBeGreaterThan(0)
      await expectLiveCaptureScene(games, demo.id, capturePosition)

      await expectNaturalDetonation(games, demo, beforePlant, punchFloors, movedRevisions)

      await waitClear(games[0])
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 4, row: 1 })
      const resumedDemo = await plantDemoBomb(one, games[0])
      const resumedPunch = await punchBombWest(one, games, resumedDemo.demo, resumedDemo.plant)
      const resumedAction = await resumePunchedBombAndSendEquipment(
        one, games, resumedDemo.demo, resumedPunch.punch)
      await moveSeatTo(one, games[0], 'one', { column: 4, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
      await moveSeatTo(one, games[0], 'one', { column: 3, row: 2 })
      await expectNaturalDetonation(games, resumedDemo.demo, resumedDemo.beforePlant,
        resumedPunch.punchFloors, resumedPunch.movedRevisions)
      await confirmResumedEquipmentAccepted(one, games[0], resumedAction)
    } finally {
      await Promise.all(contexts.map((context) => context.close()))
    }
  })
