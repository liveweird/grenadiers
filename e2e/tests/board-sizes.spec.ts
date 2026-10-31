import { expect, test, type Browser, type BrowserContextOptions, type Page } from '@playwright/test'
import type { BoardSize, Seat } from '../../web/src/game/v17/schema'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor,
  type ObservedGame } from './support/duel'
import { admissionBaseline, disconnectTransport, installTransportProbe } from './support/reconnect'

type Controller = 'human' | 'cpu'
type Difficulty = 'easy' | 'standard'
type Group = { pages: Page[]; games: ObservedGame[]; close: () => Promise<void> }

const BOARDS = [
  { width: 15, height: 13 }, { width: 13, height: 11 }, { width: 11, height: 9 },
] as const satisfies readonly BoardSize[]
const COUNTS = [2, 3, 4] as const
const DIFFICULTIES = ['easy', 'standard'] as const
const SEATS = ['one', 'two', 'three', 'four'] as const
const boardValue = ({ width, height }: BoardSize): string => `${width}x${height}`

const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

async function createGroup(
  browser: Browser, count: number, lastOptions?: BrowserContextOptions, probeLast = false,
): Promise<Group> {
  const contexts = await Promise.all(Array.from({ length: count }, (_, index) =>
    browser.newContext(index === count - 1 ? lastOptions : undefined)))
  const pages = await Promise.all(contexts.map((context) => context.newPage()))
  const games = pages.map(observeGame)
  if (probeLast) await installTransportProbe(pages.at(-1)!)
  await Promise.all(pages.map((page) => page.goto('/')))
  return { pages, games, close: async () => { await Promise.all(contexts.map((context) => context.close())) } }
}

async function createCustom(
  page: Page, board: BoardSize, controllers: readonly Controller[], difficulty: Difficulty = 'standard',
): Promise<string | null> {
  await page.getByRole('combobox', { name: 'Players', exact: true }).selectOption(String(controllers.length))
  await page.getByRole('combobox', { name: 'Board size', exact: true }).selectOption(boardValue(board))
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
  const invitation = page.getByLabel('Room invitation code', { exact: true })
  if (!controllers.slice(1).includes('human')) {
    await expect(invitation).toHaveCount(0)
    return null
  }
  await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  return invitation.innerText()
}

async function join(page: Page, code: string): Promise<void> {
  await page.getByRole('textbox', { name: 'Room code', exact: true }).fill(code)
  await page.getByRole('button', { name: 'Join room', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Players', exact: true })).toContainText('(you)')
}

async function joinInSeatOrder(group: Group, code: string): Promise<void> {
  for (const [index, page] of group.pages.slice(1).entries()) {
    await join(page, code)
    await expect.poll(() => group.games[index + 1].frames
      .filter((frame) => frame.type === 'admitted').at(-1)?.youSeat).toBe(SEATS[index + 1])
  }
}

async function startRound(pages: readonly Page[], games: readonly ObservedGame[]): Promise<void> {
  await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
  await Promise.all(games.map((game) =>
    expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
}

function expectBoard(game: ObservedGame, board: BoardSize, count: number): void {
  const snapshot = latestSnapshot(game)
  expect(snapshot?.room).toMatchObject({
    rulesVersion: 9,
    mapId: 'generated_v2',
    configuration: { environment: {
      stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
    } },
    settings: { environment: {
      stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
    } },
  })
  expect(snapshot?.room.configuration.board).toEqual(board)
  expect(snapshot?.state.arena).toMatchObject({ width: board.width, height: board.height })
  expect(snapshot?.state.arena.terrain).toHaveLength(board.width * board.height)
  expect(snapshot?.state.actors).toHaveLength(count)
  expect(snapshot?.state.stormCloud).not.toBeNull()
  expect(snapshot?.state.lairs).toHaveLength(2)
  expect(snapshot?.room.availableItems).toHaveLength(22)
  expect(snapshot?.room.settings.enabledItems).toEqual(snapshot?.room.availableItems)
}

async function moveOwnSeat(page: Page, game: ObservedGame, seat: Seat, board: BoardSize): Promise<void> {
  const actualSeat = game.frames.filter((frame) => frame.type === 'admitted').at(-1)?.youSeat
  expect(actualSeat).toBe(seat)
  const snapshot = latestSnapshot(game)!
  const actor = visibleActor(snapshot, seat)
  const before = actor?.position.x
  expect(before).toBeDefined()
  const east = before! < board.width * 512
  const actorCell = { column: Math.floor(before! / 1024), row: Math.floor(actor!.position.y / 1024) }
  const neighbor = { column: actorCell.column + (east ? 1 : -1), row: actorCell.row }
  expect(snapshot.state.arena.terrain[neighbor.row * board.width + neighbor.column]).toBe('floor')
  expect(snapshot.state.arena.blocks).not.toContainEqual(neighbor)
  const courtyard = page.getByRole('region', { name: 'Grenadiers courtyard' })
  await expect(page.locator('.game-arena canvas')).toBeVisible()
  await expect(courtyard).toHaveAttribute('tabindex', '0')
  await courtyard.click()
  const commandCount = game.commands.length
  const key = east ? 'ArrowRight' : 'ArrowLeft'
  const direction = east ? 'east' : 'west'
  await page.keyboard.down(key)
  try {
    await expect.poll(() => game.commands.slice(commandCount).some((command) =>
      command.type === 'movement' && command.direction === direction), { intervals: [20] }).toBe(true)
    await expect.poll(() => {
      const current = visibleActor(latestSnapshot(game), seat)?.position.x
      return current !== undefined && (east ? current > before! + 128 : current < before! - 128)
    }, { intervals: [20] }).toBe(true)
  } finally { await page.keyboard.up(key) }
}

async function expectCpuProgress(game: ObservedGame): Promise<void> {
  const initial = latestSnapshot(game)!
  const cpuIds = new Set(initial.room.seats
    .filter(({ controller }) => controller.type === 'cpu')
    .map(({ seat }) => visibleActor(initial, seat)?.id)
    .filter((id): id is number => id !== undefined))
  expect(cpuIds.size).toBe(initial.room.seats.filter(({ controller }) => controller.type === 'cpu').length)
  const positions = new Map(initial.state.actors.filter((actor) => cpuIds.has(actor.id))
    .map((actor) => [actor.id, actor.type === 'visible' ? `${actor.position.x},${actor.position.y}` : 'hidden']))
  const snapshots = () => game.frames.filter((frame) => frame.type === 'snapshot')
  await expect.poll(() => [...cpuIds].every((id) => snapshots().some((frame) =>
    frame.state.bombs.some(({ owner }) => owner === id)
      || frame.state.actors.some((actor) => actor.id === id && actor.type === 'visible'
        && positions.get(id) !== `${actor.position.x},${actor.position.y}`))),
  { timeout: 12_000 }).toBe(true)
  await expect.poll(() => snapshots().some((frame) =>
    frame.state.bombs.some(({ owner }) => cpuIds.has(owner))), { timeout: 12_000 }).toBe(true)
}

test('all human controller counts run on every board preset', async ({ browser }) => {
  test.setTimeout(180_000)
  for (const board of BOARDS) for (const count of COUNTS) {
    await test.step(`${boardValue(board)} with ${count} humans`, async () => {
      const group = await createGroup(browser, count)
      try {
        const controllers = Array.from({ length: count }, () => 'human' as const)
        const code = await createCustom(group.pages[0], board, controllers)
        expect(code).not.toBeNull()
        await expect(group.pages[0].getByText(`Board: ${board.width} × ${board.height}`, { exact: true }))
          .toBeVisible()
        await joinInSeatOrder(group, code!)
        await startRound(group.pages, group.games)
        for (const game of group.games) expectBoard(game, board, count)
        await moveOwnSeat(group.pages.at(-1)!, group.games.at(-1)!, SEATS[count - 1], board)
      } finally { await group.close() }
    })
  }
})

test('CPU controller counts and difficulties make authoritative progress on every board preset', async ({ browser }) => {
  test.setTimeout(240_000)
  for (const board of BOARDS) for (const count of COUNTS) for (const difficulty of DIFFICULTIES) {
    await test.step(`${boardValue(board)} with ${count - 1} ${difficulty} CPUs`, async () => {
      const group = await createGroup(browser, 1)
      try {
        const controllers: Controller[] = ['human', ...Array.from({ length: count - 1 }, () => 'cpu' as const)]
        await createCustom(group.pages[0], board, controllers, difficulty)
        expect(latestRoom(group.games[0])?.seats.slice(1).every(({ ready, controller }) =>
          ready && controller.type === 'cpu' && controller.difficulty === difficulty)).toBe(true)
        await startRound(group.pages, group.games)
        expectBoard(group.games[0], board, count)
        await expectCpuProgress(group.games[0])
      } finally { await group.close() }
    })
  }
})

test('quick duel and solo retain the default board', async ({ browser }) => {
  const group = await createGroup(browser, 1)
  try {
    await group.pages[0].getByRole('combobox', { name: 'Board size', exact: true }).selectOption('11x9')
    await group.pages[0].getByRole('button', { name: 'Create room', exact: true }).click()
    await expect.poll(() => latestRoom(group.games[0])?.configuration.board).toEqual({ width: 15, height: 13 })
    const duelRoomId = latestRoom(group.games[0])!.roomId
    await group.pages[0].getByRole('button', { name: 'Leave room', exact: true }).click()
    await expect(group.pages[0].getByRole('button', { name: 'Play solo against CPU', exact: true })).toBeVisible()
    await group.pages[0].getByRole('combobox', { name: 'Board size', exact: true }).selectOption('11x9')
    const admissionCount = group.games[0].frames.filter((frame) => frame.type === 'admitted').length
    await group.pages[0].getByRole('button', { name: 'Play solo against CPU', exact: true }).click()
    await expect.poll(() => group.games[0].frames.filter((frame) => frame.type === 'admitted').length)
      .toBeGreaterThan(admissionCount)
    const solo = group.games[0].frames.findLast((frame) => frame.type === 'admitted')!
    expect(solo.room.roomId).not.toBe(duelRoomId)
    expect(solo.room.configuration.controllers).toEqual(['human', 'cpu'])
    expect(solo.room.configuration.board).toEqual({ width: 15, height: 13 })
  } finally { await group.close() }
})

test('a small four-human room reconnects and rematches without changing geometry', async ({ browser }) => {
  test.setTimeout(120_000)
  const board = { width: 11, height: 9 } as const
  const group = await createGroup(browser, 4, {
    viewport: { width: 320, height: 844 }, reducedMotion: 'reduce',
  }, true)
  try {
    const code = await createCustom(group.pages[0], board, ['human', 'human', 'human', 'human'])
    await joinInSeatOrder(group, code!)
    await group.pages[0].getByLabel('Round wins needed').selectOption('1')
    await group.pages[0].getByLabel('Bomb fuse').selectOption('120')
    await group.pages[0].getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => latestRoom(group.games[3])?.settings.winsToFinish).toBe(1)
    await startRound(group.pages, group.games)
    await moveOwnSeat(group.pages[3], group.games[3], 'four', board)
    await expectPaintedCanvas(group.pages[3])
    const readGeometry = () => group.pages[3].locator('.game-arena canvas').evaluate((canvas) => {
      const bounds = canvas.getBoundingClientRect()
      return {
        width: (canvas as HTMLCanvasElement).width, height: (canvas as HTMLCanvasElement).height,
        cssWidth: bounds.width, cssHeight: bounds.height, dpr: devicePixelRatio,
        overflow: document.documentElement.scrollWidth > innerWidth,
      }
    })
    const tileGeometry = await readGeometry()
    // The 3D camera fits the board inside the viewport; its ground tiles are foreshortened.
    // Check backing-store/CSS agreement, rather than the old flat-board tile aspect ratio.
    expect(tileGeometry.width).toBeGreaterThan(0)
    expect(Math.abs(tileGeometry.width - Math.round(tileGeometry.cssWidth) * tileGeometry.dpr))
      .toBeLessThanOrEqual(1)
    expect(Math.abs(tileGeometry.height - Math.round(tileGeometry.cssHeight) * tileGeometry.dpr))
      .toBeLessThanOrEqual(1)
    expect(tileGeometry.overflow).toBe(false)

    const admission = await admissionBaseline(group.pages[3])
    expect(admission).not.toBeNull()
    const revisionBeforeReconnect = latestSnapshot(group.games[3])!.room.revision
    await disconnectTransport(group.pages[3])
    await expect.poll(async () => (await admissionBaseline(group.pages[3]))?.index ?? 0)
      .toBeGreaterThan(admission!.index)
    await expect.poll(() => latestSnapshot(group.games[3])?.room.revision ?? 0)
      .toBeGreaterThan(revisionBeforeReconnect)
    expect(await admissionBaseline(group.pages[3])).toMatchObject({
      roomId: admission!.roomId, seat: 'four', hasState: true, stableCapability: true,
    })
    expectBoard(group.games[3], board, 4)

    await Promise.all(group.pages.slice(0, 3).map(async (page) => {
      await page.getByRole('region', { name: 'Grenadiers courtyard' }).click()
      await page.keyboard.press('Space')
    }))
    await Promise.all(group.games.map((game) =>
      expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('matchResult')))
    const completed = latestSnapshot(group.games[0])!
    expect(completed.room.phase).toMatchObject({
      type: 'matchResult', result: { type: 'winner', seat: 'four', reason: 'roundWins' },
    })
    expect(completed.room.scores).toEqual([
      { seat: 'one', wins: 0 }, { seat: 'two', wins: 0 },
      { seat: 'three', wins: 0 }, { seat: 'four', wins: 1 },
    ])
    expect(completed.state.actors.map(({ seat, alive }) => ({ seat, alive }))).toEqual([
      { seat: 'one', alive: false }, { seat: 'two', alive: false },
      { seat: 'three', alive: false }, { seat: 'four', alive: true },
    ])
    const completedRound = completed.room.roundId
    await Promise.all(group.pages.map(async (page) => {
      const rematch = page.getByRole('button', { name: 'Play again', exact: true })
      await expect(rematch).toBeEnabled({ timeout: 10_000 })
      await rematch.click()
    }))
    await Promise.all(group.games.map((game) =>
      expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')))
    for (const game of group.games) {
      const room = latestRoom(game)!
      expect(room.configuration.board).toEqual(board)
      expect(room.roundId).toBe(completedRound + 1)
      expect(room.scores).toEqual(SEATS.map((seat) => ({ seat, wins: 0 })))
    }
    await startRound(group.pages, group.games)
    for (const game of group.games) expectBoard(game, board, 4)
    await expect.poll(readGeometry).toEqual(tileGeometry)
  } finally { await group.close() }
})
