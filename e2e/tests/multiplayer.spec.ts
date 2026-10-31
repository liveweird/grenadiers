import { expect, test, type Browser, type Page } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from './support/duel'
import { admissionBaseline, disconnectTransport, installTransportProbe } from './support/reconnect'

type Controller = 'human' | 'cpu'
type Difficulty = 'easy' | 'standard'
type Seat = 'one' | 'two' | 'three' | 'four'

const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

async function createPages(
  browser: Browser,
  count: number,
  transportProbeSeats: readonly number[] = [],
): Promise<{ pages: Page[]; games: ObservedGame[]; close: () => Promise<void> }> {
  const contexts = await Promise.all(Array.from({ length: count }, () => browser.newContext()))
  const pages = await Promise.all(contexts.map((context) => context.newPage()))
  const games = pages.map(observeGame)
  await Promise.all(transportProbeSeats.map((index) => installTransportProbe(pages[index]!)))
  await Promise.all(pages.map((page) => page.goto('/')))
  return { pages, games, close: async () => { await Promise.all(contexts.map((context) => context.close())) } }
}

async function createCustom(
  page: Page,
  controllers: readonly Controller[],
  difficulty: Difficulty = 'standard',
): Promise<string | null> {
  await page.getByRole('combobox', { name: 'Players', exact: true }).selectOption(String(controllers.length))
  for (const [index, controller] of controllers.entries()) {
    if (index > 0) {
      await page.getByRole('combobox', { name: `Player ${index + 1} controller`, exact: true })
        .selectOption(controller)
    }
  }
  if (controllers.includes('cpu')) {
    await page.getByRole('combobox', { name: 'CPU difficulty', exact: true }).last().selectOption(difficulty)
  }
  await page.getByRole('button', { name: 'Create custom match', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Players', exact: true })).toBeVisible()
  if (!controllers.slice(1).includes('human')) return null
  const invitation = page.getByLabel('Room invitation code', { exact: true })
  await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  return invitation.innerText()
}

async function join(page: Page, code: string, invitationVisible: boolean): Promise<void> {
  await page.getByRole('textbox', { name: 'Room code', exact: true }).fill(code)
  await page.getByRole('button', { name: 'Join room', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Players', exact: true })).toContainText('(you)')
  const invitation = page.getByLabel('Room invitation code', { exact: true })
  if (invitationVisible) await expect(invitation).toHaveText(code)
  else await expect(invitation).toHaveCount(0)
}

async function expectPlayers(page: Page, count: number): Promise<void> {
  const list = page.getByRole('list', { name: 'Players', exact: true })
  for (let index = 1; index <= count; index += 1) {
    await expect(list).toContainText(`Player ${['one', 'two', 'three', 'four'][index - 1]}`)
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
}

async function startRound(pages: readonly Page[], games: readonly ObservedGame[]): Promise<void> {
  await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
  await Promise.all(games.map((game) =>
    expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
}

async function moveOwnSeat(
  page: Page,
  game: ObservedGame,
  seat: Seat,
  key: 'ArrowLeft' | 'ArrowRight',
  direction: 'west' | 'east',
): Promise<void> {
  const before = visibleActor(latestSnapshot(game), seat)?.position.x
  expect(before).toBeDefined()
  await page.getByRole('region', { name: 'Grenadiers courtyard' }).click()
  await page.keyboard.down(key)
  try {
    await expect.poll(() => game.commands.some((command) =>
      command.type === 'movement' && command.direction === direction)).toBe(true)
    await expect.poll(() => {
      const current = visibleActor(latestSnapshot(game), seat)?.position.x
      if (current === undefined || before === undefined) return false
      return direction === 'east' ? current > before + 128 : current < before - 128
    }, { intervals: [20] }).toBe(true)
  } finally {
    await page.keyboard.up(key)
  }
}

test('three humans join an explicit three-seat match', async ({ browser }) => {
  test.setTimeout(60_000)
  const group = await createPages(browser, 3, [2])
  try {
    const code = await createCustom(group.pages[0], ['human', 'human', 'human'])
    expect(code).not.toBeNull()
    await join(group.pages[1], code!, true)
    await join(group.pages[2], code!, true)
    await expectPlayers(group.pages[0], 3)
    await startRound(group.pages, group.games)
    const running = latestSnapshot(group.games[2])!
    expect(running.room.configuration).toEqual({
      controllers: ['human', 'human', 'human'], cpuDifficulty: null,
      board: { width: 15, height: 13 },
      environment: {
        stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 }, spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 },
      },
    })
    expect(running.room.seats.map(({ seat, controller, occupied }) =>
      ({ seat, controller, occupied }))).toEqual([
      { seat: 'one', controller: { type: 'human' }, occupied: true },
      { seat: 'two', controller: { type: 'human' }, occupied: true },
      { seat: 'three', controller: { type: 'human' }, occupied: true },
    ])
    await moveOwnSeat(group.pages[2], group.games[2], 'three', 'ArrowLeft', 'west')

    const admission = await admissionBaseline(group.pages[2])
    expect(admission).toMatchObject({ seat: 'three', hasState: false })
    await disconnectTransport(group.pages[2])
    await expect.poll(async () => (await admissionBaseline(group.pages[2]))?.index ?? 0)
      .toBeGreaterThan(admission!.index)
    expect(await admissionBaseline(group.pages[2])).toMatchObject({
      roomId: running.room.roomId, seat: 'three', hasState: true, stableCapability: true,
    })
    await expect.poll(() => latestSnapshot(group.games[2])?.room.phase.type).toBe('running')
  } finally { await group.close() }
})

test('four humans share the four-seat lobby at 320 pixels', async ({ browser }) => {
  test.setTimeout(60_000)
  const group = await createPages(browser, 4)
  try {
    const code = await createCustom(group.pages[0], ['human', 'human', 'human', 'human'])
    expect(code).not.toBeNull()
    await join(group.pages[1], code!, true)
    await join(group.pages[2], code!, true)
    await join(group.pages[3], code!, true)
    await group.pages[3].setViewportSize({ width: 320, height: 844 })
    await expectPlayers(group.pages[3], 4)

    const settingsRevision = latestRoom(group.games[0])!.settingsRevision
    await group.pages[0].getByLabel('Round wins needed').selectOption('1')
    await group.pages[0].getByLabel('Bomb fuse').selectOption('120')
    await group.pages[0].getByRole('button', { name: 'Apply rules', exact: true }).click()
    await Promise.all(group.games.map((game) => expect.poll(() => {
      const room = latestRoom(game)
      return room && room.settingsRevision > settingsRevision
        ? { winsToFinish: room.settings.winsToFinish, bombFuseTicks: room.settings.bombFuseTicks }
        : null
    }).toEqual({ winsToFinish: 1, bombFuseTicks: 120 })))

    await startRound(group.pages, group.games)
    const running = latestSnapshot(group.games[3])!
    expect(running.room.configuration).toEqual({
      controllers: ['human', 'human', 'human', 'human'], cpuDifficulty: null,
      board: { width: 15, height: 13 },
      environment: {
        stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 }, spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 },
      },
    })
    expect(running.state.actors.map(({ seat }) => seat)).toEqual(['one', 'two', 'three', 'four'])
    const initialCells = running.state.actors.map((actor) => {
      expect(actor.type).toBe('visible')
      if (actor.type !== 'visible') throw new Error(`Initial actor ${actor.seat} is not visible`)
      return {
        column: Math.floor(actor.position.x / 1024),
        row: Math.floor(actor.position.y / 1024),
      }
    }).sort((left, right) => left.row - right.row || left.column - right.column)
    expect(initialCells).toEqual([
      { column: 1, row: 1 }, { column: 13, row: 1 },
      { column: 1, row: 11 }, { column: 13, row: 11 },
    ])
    await moveOwnSeat(group.pages[3], group.games[3], 'four', 'ArrowRight', 'east')
    expect(await group.pages[3].evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)

    await Promise.all(group.pages.slice(0, 3).map(async (page) => {
      await page.getByRole('region', { name: 'Grenadiers courtyard' }).click()
      await page.keyboard.press('Space')
    }))
    await Promise.all(group.games.map((game) =>
      expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('matchResult')))
    const finalRoundId = latestSnapshot(group.games[0])!.room.roundId
    const expectedScores = [
      { seat: 'one', wins: 0 }, { seat: 'two', wins: 0 },
      { seat: 'three', wins: 0 }, { seat: 'four', wins: 1 },
    ]
    for (const game of group.games) {
      const result = latestSnapshot(game)!
      expect(result.room.phase).toMatchObject({
        type: 'matchResult', result: { type: 'winner', seat: 'four', reason: 'roundWins' },
      })
      expect(result.room.scores).toEqual(expectedScores)
      expect(result.state.actors.map(({ seat, alive }) => ({ seat, alive }))).toEqual([
        { seat: 'one', alive: false }, { seat: 'two', alive: false },
        { seat: 'three', alive: false }, { seat: 'four', alive: true },
      ])
    }

    await Promise.all(group.pages.map((page) =>
      expect(page.getByRole('button', { name: 'Play again', exact: true })).toBeVisible()))
    await group.pages[3].getByRole('button', { name: 'Play again', exact: true }).click()
    await expect.poll(() => latestSnapshot(group.games[0])?.room.seats[3]?.rematchReady).toBe(true)
    expect(latestSnapshot(group.games[0])?.room.phase.type).toBe('matchResult')
    await Promise.all(group.pages.slice(0, 3).map((page) =>
      page.getByRole('button', { name: 'Play again', exact: true }).click()))
    await Promise.all(group.games.map((game) => expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')))
    for (const game of group.games) {
      const reset = latestRoom(game)!
      expect(reset).toMatchObject({ roundId: finalRoundId + 1, roundNumber: 0, lastRound: null })
      expect(reset.scores).toEqual([
        { seat: 'one', wins: 0 }, { seat: 'two', wins: 0 },
        { seat: 'three', wins: 0 }, { seat: 'four', wins: 0 },
      ])
      expect(reset.seats.map(({ ready, rematchReady }) => ({ ready, rematchReady }))).toEqual([
        { ready: false, rematchReady: false }, { ready: false, rematchReady: false },
        { ready: false, rematchReady: false }, { ready: false, rematchReady: false },
      ])
    }

    await startRound(group.pages, group.games)
    await Promise.all(group.games.map((game) => expect.poll(() =>
      latestSnapshot(game)?.state.actors.map(({ alive }) => alive)).toEqual([true, true, true, true])))
  } finally { await group.close() }
})

test('human CPU human assigns both human seats without exposing a CPU join', async ({ browser }) => {
  test.setTimeout(60_000)
  const group = await createPages(browser, 2)
  try {
    const code = await createCustom(group.pages[0], ['human', 'cpu', 'human'], 'easy')
    expect(code).not.toBeNull()
    await join(group.pages[1], code!, false)
    await expectPlayers(group.pages[0], 3)
    await expect(group.pages[0].getByText('CPU difficulty: Easy', { exact: true })).toBeVisible()
    expect(latestRoom(group.games[0])?.seats[1]).toMatchObject({
      seat: 'two', occupied: true, ready: true,
      controller: { type: 'cpu', difficulty: 'easy' }, connection: { type: 'serverControlled' },
    })
    await startRound(group.pages, group.games)
    const running = latestSnapshot(group.games[0])!
    expect(running.room.configuration).toEqual({
      controllers: ['human', 'cpu', 'human'], cpuDifficulty: 'easy',
      board: { width: 15, height: 13 },
      environment: {
        stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 }, spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 },
      },
    })
    expect(running.room.seats[1]).toMatchObject({
      seat: 'two', occupied: true, ready: false,
      controller: { type: 'cpu', difficulty: 'easy' }, connection: { type: 'serverControlled' },
    })
    expect(running.room.seats[2]).toMatchObject({
      seat: 'three', occupied: true, controller: { type: 'human' },
    })
  } finally { await group.close() }
})

test('one human and three CPUs creates a ready four-controller lobby', async ({ browser }) => {
  test.setTimeout(60_000)
  const group = await createPages(browser, 1)
  try {
    const code = await createCustom(group.pages[0], ['human', 'cpu', 'cpu', 'cpu'])
    expect(code).toBeNull()
    await expect(group.pages[0].getByLabel('Room invitation code', { exact: true })).toHaveCount(0)
    await expectPlayers(group.pages[0], 4)
    await expect(group.pages[0].getByRole('button', { name: 'Ready', exact: true })).toBeEnabled()
    expect(latestRoom(group.games[0])?.seats.slice(1).map(({ ready }) => ready)).toEqual([true, true, true])
    await startRound(group.pages, group.games)
    const running = latestSnapshot(group.games[0])!
    expect(running.room.configuration).toEqual({
      controllers: ['human', 'cpu', 'cpu', 'cpu'], cpuDifficulty: 'standard',
      board: { width: 15, height: 13 },
      environment: {
        stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 }, spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 },
      },
    })
    expect(running.room.seats.slice(1).map(({ ready, controller, connection }) =>
      ({ ready, controller, connection }))).toEqual([
      { ready: false, controller: { type: 'cpu', difficulty: 'standard' }, connection: { type: 'serverControlled' } },
      { ready: false, controller: { type: 'cpu', difficulty: 'standard' }, connection: { type: 'serverControlled' } },
      { ready: false, controller: { type: 'cpu', difficulty: 'standard' }, connection: { type: 'serverControlled' } },
    ])
  } finally { await group.close() }
})

test('three CPUs escape their first bombs in a default four-player match', async ({ browser }) => {
  const group = await createPages(browser, 1)
  try {
    await createCustom(group.pages[0], ['human', 'cpu', 'cpu', 'cpu'])
    await startRound(group.pages, group.games)
    const game = group.games[0]!
    const roundId = latestSnapshot(game)!.room.roundId
    for (const seat of ['two', 'three', 'four'] as const) {
      const owner = visibleActor(latestSnapshot(game), seat)!.id
      const firstPlacement = () => game.frames
        .flatMap((frame) => frame.type === 'snapshot' && frame.room.roundId === roundId ? frame.events : [])
        .find((event) => event.kind.type === 'bombPlaced' && event.kind.bomb.owner === owner)
      await expect.poll(firstPlacement, { timeout: 10_000 }).toBeTruthy()
      const placed = firstPlacement()!
      if (placed.kind.type !== 'bombPlaced') throw new Error('Expected a CPU bomb placement')
      const bombId = placed.kind.bomb.id
      const detonation = () => game.frames.find((frame) =>
        frame.type === 'snapshot' && frame.room.roundId === roundId
          && frame.events.some((event) =>
            event.kind.type === 'bombDetonated' && event.kind.bomb.id === bombId))
      await expect.poll(detonation, { timeout: 10_000 }).toBeTruthy()
      const exploded = detonation()!
      if (exploded.type !== 'snapshot') throw new Error('Expected the detonation snapshot')
      expect(visibleActor(exploded, seat)?.alive).toBe(true)
      await expect.poll(() => latestSnapshot(game)?.room.tick).toBeGreaterThan(exploded.room.tick + 30)
      const eliminatedDuringBlast = game.frames.some((frame) =>
        frame.type === 'snapshot' && frame.room.roundId === roundId
          && frame.events.some((event) => event.kind.type === 'actorEliminated'
            && event.kind.seat === seat && event.tick <= exploded.room.tick + 30))
      expect(eliminatedDuringBlast).toBe(false)
    }
  } finally { await group.close() }
})

test('three CPUs keep moving with all optional features disabled', async ({ browser }) => {
  const group = await createPages(browser, 1)
  try {
    await createCustom(group.pages[0], ['human', 'cpu', 'cpu', 'cpu'])
    const game = group.games[0]!
    const revision = latestRoom(game)!.settingsRevision
    await group.pages[0].getByRole('button', { name: 'All off', exact: true }).click()
    await group.pages[0].getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => latestRoom(game)?.settingsRevision).toBeGreaterThan(revision)
    expect(latestRoom(game)?.settings.environment).toMatchObject({
      stormCloud: false, lairs: { enabled: false, maxLivingMobs: 1 },
    })
    await startRound(group.pages, group.games)
    const running = latestSnapshot(game)!
    const seats = ['two', 'three', 'four'] as const
    const initial = seats.map((seat) => ({ ...visibleActor(running, seat)!.position }))
    await expect.poll(() => seats.every((seat, index) => game.frames.some((frame) => {
      if (frame.type !== 'snapshot' || frame.room.roundId !== running.room.roundId) return false
      const current = visibleActor(frame, seat)
      return current && (current.position.x !== initial[index]!.x || current.position.y !== initial[index]!.y)
    }))).toBe(true)
    expect(latestSnapshot(game)?.state.lairs).toEqual([])
    expect(latestSnapshot(game)?.state.mobs).toEqual([])
  } finally { await group.close() }
})
