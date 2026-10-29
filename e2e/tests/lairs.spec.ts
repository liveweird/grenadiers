import { expect, test, type Browser, type Page } from '@playwright/test'
import type { GameServerMessage, MobView } from '../../web/src/game/v16/schema'
import { createResponsiveBrowserContexts } from './support/browser-contexts'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from './support/duel'
import { admissionBaseline, disconnectTransport, installTransportProbe } from './support/reconnect'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type Group = { pages: [Page, Page]; games: [ObservedGame, ObservedGame]; close: () => Promise<void> }

const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')

async function createGroup(browser: Browser, responsive = true): Promise<Group> {
  const contexts = responsive
    ? await createResponsiveBrowserContexts(browser)
    : await Promise.all([browser.newContext(), browser.newContext()])
  const pages = await Promise.all(contexts.map((context) => context.newPage())) as [Page, Page]
  await installTransportProbe(pages[1])
  const games = pages.map(observeGame) as [ObservedGame, ObservedGame]
  await Promise.all(pages.map((page) => page.goto('/')))
  return { pages, games, close: async () => { await Promise.all(contexts.map((context) => context.close())) } }
}

async function joinRoom(group: Group, polishPeer: boolean): Promise<void> {
  const [one, two] = group.pages
  await one.getByRole('button', { name: 'Create room', exact: true }).click()
  await expect.poll(() => latestRoom(group.games[0])).toBeTruthy()
  const code = latestRoom(group.games[0])!.roomCode
  if (polishPeer) await two.getByText('Polski', { exact: true }).click()
  await two.getByRole('textbox', { name: polishPeer ? 'Kod pokoju' : 'Room code', exact: true }).fill(code)
  await two.getByRole('button', { name: polishPeer ? 'Dołącz do pokoju' : 'Join room', exact: true }).click()
  await expect.poll(() => latestRoom(group.games[1])?.roomId).toBe(latestRoom(group.games[0])?.roomId)
}

async function startRound(group: Group, polishPeer: boolean): Promise<void> {
  await Promise.all([
    group.pages[0].getByRole('button', { name: 'Ready', exact: true }).click(),
    group.pages[1].getByRole('button', { name: polishPeer ? 'Gotowy' : 'Ready', exact: true }).click(),
  ])
  await Promise.all(group.games.map((game) =>
    expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
}

async function canvasDifference(
  page: Page, first: Buffer, second: Buffer, board: { width: number; height: number },
): Promise<number> {
  return page.evaluate(async ({ one, two, width, height }) => {
    const decode = async (encoded: string): Promise<HTMLImageElement> => {
      const image = new Image()
      image.src = `data:image/png;base64,${encoded}`
      await image.decode()
      return image
    }
    const [before, after] = await Promise.all([decode(one), decode(two)])
    const canvas = document.createElement('canvas')
    canvas.width = before.width
    canvas.height = before.height
    const context = canvas.getContext('2d', { willReadFrequently: true })
    if (!context || after.width !== before.width || after.height !== before.height) return 0
    const cellWidth = canvas.width / width
    const cellHeight = canvas.height / height
    const x = Math.floor(cellWidth * (width / 2 - 3))
    const y = Math.floor(cellHeight * (height / 2 - 3))
    const regionWidth = Math.ceil(cellWidth * 6)
    const regionHeight = Math.ceil(cellHeight * 6)
    context.drawImage(before, 0, 0)
    const left = context.getImageData(x, y, regionWidth, regionHeight).data
    context.clearRect(0, 0, canvas.width, canvas.height)
    context.drawImage(after, 0, 0)
    const right = context.getImageData(x, y, regionWidth, regionHeight).data
    let changed = 0
    for (let index = 0; index < left.length; index += 4) {
      if (left[index] !== right[index] || left[index + 1] !== right[index + 1]
        || left[index + 2] !== right[index + 2] || left[index + 3] !== right[index + 3]) changed += 1
    }
    return changed
  }, { one: first.toString('base64'), two: second.toString('base64'), ...board })
}

async function lairPillarDifference(page: Page, image: Buffer, snapshot: Snapshot): Promise<number> {
  const lair = snapshot.state.lairs[0]!
  const width = snapshot.state.arena.width
  const ordinaryPillar = { column: 2, row: 2 }
  expect(snapshot.state.arena.terrain[ordinaryPillar.row * width + ordinaryPillar.column]).toBe('wall')
  expect(snapshot.state.lairs.map(({ cell }) => cell)).not.toContainEqual(ordinaryPillar)
  return page.evaluate(async ({ encoded, board, first, second }) => {
    const image = new Image()
    image.src = `data:image/png;base64,${encoded}`
    await image.decode()
    const canvas = document.createElement('canvas')
    canvas.width = image.width
    canvas.height = image.height
    const context = canvas.getContext('2d', { willReadFrequently: true })
    if (!context) return 0
    context.drawImage(image, 0, 0)
    const cellWidth = canvas.width / board.width
    const cellHeight = canvas.height / board.height
    const patch = (cell: { column: number; row: number }) => context.getImageData(
      Math.floor((cell.column + 0.2) * cellWidth), Math.floor((cell.row + 0.2) * cellHeight),
      Math.max(1, Math.floor(cellWidth * 0.6)), Math.max(1, Math.floor(cellHeight * 0.6)),
    ).data
    const left = patch(first)
    const right = patch(second)
    let changed = 0
    for (let index = 0; index < left.length; index += 4) {
      if (left[index] !== right[index] || left[index + 1] !== right[index + 1]
        || left[index + 2] !== right[index + 2] || left[index + 3] !== right[index + 3]) changed += 1
    }
    return changed
  }, {
    encoded: image.toString('base64'),
    board: { width, height: snapshot.state.arena.height }, first: lair.cell,
    second: ordinaryPillar,
  })
}

function expectPrivateNavigation(mob: MobView): void {
  expect(Object.keys(mob).sort()).toEqual(['frozen', 'heading', 'id', 'lairId', 'position'])
  expect(mob).not.toHaveProperty('target')
  expect(mob).not.toHaveProperty('path')
}

test('enabled lairs spawn capped private moving mobs, render on both clients, reconnect and rematch',
  async ({ browser }, testInfo) => {
    test.setTimeout(90_000)
    const group = await createGroup(browser)
    try {
      await joinRoom(group, true)
      const [one, two] = group.pages
      await one.getByRole('checkbox', { name: 'Storm cloud', exact: true }).uncheck()
      await one.getByLabel('Spawn interval (seconds)', { exact: true }).fill('5')
      await one.getByLabel('Living monsters per lair', { exact: true }).fill('1')
      await one.getByLabel('Round wins needed').selectOption('1')
      await one.getByLabel('Bomb fuse').selectOption('60')
      await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
      const environment = {
        stormCloud: false, lairs: { enabled: true, spawnSeconds: 5, maxLivingMobs: 1 },
      }
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestRoom(game)?.settings.environment).toEqual(environment)))
      await expect(two.getByRole('checkbox', { name: 'Legowiska potworów', exact: true })).toBeChecked()
      await expect(two.getByLabel('Czas odrodzenia (sekundy)', { exact: true })).toHaveValue('5')
      await expect(two.getByLabel('Żywe potwory na legowisko', { exact: true })).toHaveValue('1')
      await startRound(group, true)

      for (const game of group.games) {
        const running = latestSnapshot(game)!
        expect(running.room).toMatchObject({ rulesVersion: 8, settings: { environment } })
        expect(running.state.stormCloud).toBeNull()
        expect(running.state.lairs.map(({ cell }) => cell)).toEqual([
          { column: 6, row: 6 }, { column: 8, row: 6 },
        ])
        expect(running.state.lairs.every(({ livingMobs }) => livingMobs === 0)).toBe(true)
      }
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestSnapshot(game)?.state.mobs.length, { timeout: 10_000 }).toBe(2)))
      const spawned = latestSnapshot(group.games[0])!
      const mob = spawned.state.mobs[0]!
      for (const game of group.games) latestSnapshot(game)!.state.mobs.forEach(expectPrivateNavigation)
      let peerAtSpawn: Snapshot | undefined
      await expect.poll(() => {
        peerAtSpawn = snapshots(group.games[1]).find((frame) =>
          frame.state.physicalTick === spawned.state.physicalTick)
        return Boolean(peerAtSpawn)
      }).toBe(true)
      expect(peerAtSpawn!.state.lairs).toEqual(spawned.state.lairs)
      expect(peerAtSpawn!.state.mobs).toEqual(spawned.state.mobs)
      const beforeCanvas = await one.locator('.game-arena canvas').screenshot()
      await expect.poll(() => {
        const current = latestSnapshot(group.games[0])?.state.mobs.find(({ id }) => id === mob.id)
        return current ? Math.abs(current.position.x - mob.position.x)
          + Math.abs(current.position.y - mob.position.y) : 0
      }).toBeGreaterThanOrEqual(256)
      await one.waitForTimeout(100)
      const afterCanvas = await one.locator('.game-arena canvas').screenshot()
      expect(await canvasDifference(one, beforeCanvas, afterCanvas, {
        width: spawned.state.arena.width, height: spawned.state.arena.height,
      })).toBeGreaterThan(20)
      expect(await lairPillarDifference(one, afterCanvas, latestSnapshot(group.games[0])!))
        .toBeGreaterThan(20)
      await Promise.all(group.pages.map(expectPaintedCanvas))

      const firstSpawnTick = spawned.state.physicalTick
      await expect.poll(() => latestSnapshot(group.games[0])?.state.physicalTick ?? 0,
        { timeout: 10_000 }).toBeGreaterThanOrEqual(firstSpawnTick + 310)
      const capped = snapshots(group.games[0]).filter((frame) =>
        frame.state.physicalTick >= firstSpawnTick && frame.state.physicalTick <= firstSpawnTick + 310)
      expect(capped.length).toBeGreaterThan(1)
      expect(capped.every((frame) => frame.state.mobs.length <= 2
        && frame.state.lairs.every(({ livingMobs }) => livingMobs <= 1))).toBe(true)

      await one.screenshot({ path: testInfo.outputPath('lairs-running-desktop-en.png'), fullPage: true })
      await two.screenshot({ path: testInfo.outputPath('lairs-running-polish-320-reduced.png'), fullPage: true })
      const admission = await admissionBaseline(two)
      expect(admission).not.toBeNull()
      const framesBeforeReconnect = group.games[1].frames.length
      await disconnectTransport(two)
      await expect.poll(async () => (await admissionBaseline(two))?.index ?? 0)
        .toBeGreaterThan(admission!.index)
      expect(await admissionBaseline(two)).toMatchObject({
        roomId: admission!.roomId, seat: 'two', hasState: true, stableCapability: true,
      })
      await expect.poll(() => group.games[1].frames.length).toBeGreaterThan(framesBeforeReconnect)
      expect(latestSnapshot(group.games[1])!.state.mobs.length).toBeGreaterThan(0)

      await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
      await one.keyboard.press('Space')
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestSnapshot(game)?.room.phase.type, { timeout: 15_000 }).toBe('matchResult')))
      await Promise.all([
        one.getByRole('button', { name: 'Play again', exact: true }).click(),
        two.getByRole('button', { name: 'Zagraj ponownie', exact: true }).click(),
      ])
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')))
      group.games.forEach((game) => expect(latestRoom(game)?.settings.environment).toEqual(environment))
      await startRound(group, true)
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestSnapshot(game)?.state.lairs).toHaveLength(2)))
    } finally { await group.close() }
  })

test('disabled lairs publish empty lair and mob arrays', async ({ browser }) => {
  const group = await createGroup(browser, false)
  try {
    await joinRoom(group, false)
    const before = latestRoom(group.games[0])!.settingsRevision
    await group.pages[0].getByRole('button', { name: 'All off', exact: true }).click()
    await group.pages[0].getByRole('button', { name: 'Apply rules', exact: true }).click()
    const environment = {
      stormCloud: false, lairs: { enabled: false, spawnSeconds: 10, maxLivingMobs: 3 },
    }
    for (const game of group.games) {
      await expect.poll(() => latestRoom(game)?.settingsRevision).toBeGreaterThan(before)
      await expect.poll(() => latestRoom(game)?.settings.environment).toEqual(environment)
    }
    await startRound(group, false)
    for (const game of group.games) {
      const state = latestSnapshot(game)!.state
      expect(state.stormCloud).toBeNull()
      expect(state.lairs).toEqual([])
      expect(state.mobs).toEqual([])
    }
  } finally { await group.close() }
})
