import { expect, test, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'

test.beforeAll(() => {
  expect(process.env.E2E_PICKUP_FIXTURE, 'Configure the documented server seed fixture before pickup acceptance')
    .toBe('arena-v1-pickups')
})

type Cell = { column: number; row: number }
const CENTER = 512
const CELL = 1024

function actor(game: ObservedGame) {
  const value = latestSnapshot(game)?.state.actors.find((candidate) => candidate.seat === 'one')
  if (!value?.alive) throw new Error('Fixture route requires a living first player')
  return value
}

async function moveTo(page: Page, game: ObservedGame, cell: Cell) {
  const target = { x: cell.column * CELL + CENTER, y: cell.row * CELL + CENTER }
  for (const axis of ['x', 'y'] as const) {
    for (let attempt = 0; attempt < 10; attempt += 1) {
      const delta = target[axis] - actor(game).position[axis]
      if (Math.abs(delta) <= 128) break
      const keys = axis === 'x' ? ['ArrowLeft', 'ArrowRight'] : ['ArrowUp', 'ArrowDown']
      const key = keys[Number(delta > 0)]
      // Bound each native input pulse: slow software rendering can deliver the
      // next snapshot after a held key has already crossed the target center.
      const duration = Math.min(80, Math.max(25, Math.abs(delta) / 3.072 / 2))
      await page.keyboard.press(key, { delay: duration })
      const stoppedRevision = latestSnapshot(game)!.room.revision
      await expect.poll(() => latestSnapshot(game)!.room.revision, { intervals: [15] })
        .toBeGreaterThan(stoppedRevision + 1)
    }
    expect(Math.abs(target[axis] - actor(game).position[axis])).toBeLessThanOrEqual(128)
  }
}

async function plant(page: Page, game: ObservedGame) {
  const previous = latestSnapshot(game)!.state.bombs.map((bomb) => bomb.id)
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)!.state.bombs.some((bomb) => !previous.includes(bomb.id))).toBe(true)
  return latestSnapshot(game)!.state.bombs.find((bomb) => !previous.includes(bomb.id))!
}

async function clearHazards(game: ObservedGame) {
  await expect.poll(() => {
    const state = latestSnapshot(game)!.state
    return state.bombs.length + state.flames.length
  }, { intervals: [30] }).toBe(0)
}

async function expectStats(pages: Page[], values: string[]) {
  for (const page of pages) {
    await expect(page.getByLabel('Player one upgrades').locator('dd')).toHaveText(values)
  }
}

test('two browsers collect generated upgrades while old bombs keep their captured range', async ({ browser }, testInfo) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 390, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    const [one, two] = pages
    const games = pages.map(observeGame)
    await Promise.all(pages.map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await one.getByRole('button', { name: 'Ready', exact: true }).click()
    await two.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('running')
    await expect.poll(() => latestSnapshot(games[1])?.room.phase.type).toBe('running')
    expect(latestSnapshot(games[0])!.room).toMatchObject({ rulesVersion: 2, mapId: 'generated_v1' })
    expect(latestSnapshot(games[0])!.state.pickups).toEqual([])
    await Promise.all(pages.map(expectPaintedCanvas))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()

    await moveTo(one, games[0], { column: 2, row: 1 })
    await plant(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await clearHazards(games[0])
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups)
      .toContainEqual({ cell: { column: 3, row: 1 }, kind: 'bombCapacity' })
    await one.screenshot({ path: testInfo.outputPath('capacity-revealed.png'), fullPage: true })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await expectStats(pages, ['2', '2', '2', '1'])

    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await plant(one, games[0])
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 2, row: 1 })
    await clearHazards(games[0])
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups)
      .toContainEqual({ cell: { column: 1, row: 3 }, kind: 'blastRange' })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    const old = await plant(one, games[0])
    expect(old.range).toBe(2)
    await moveTo(one, games[0], { column: 1, row: 3 })
    expect(actor(games[0]).bombRange).toBe(3)
    const upgraded = await plant(one, games[0])
    expect(upgraded.range).toBe(3)
    expect(upgraded.id).not.toBe(old.id)
    // Verify both live clients before compositor screenshots spend fuse time on
    // slower software renderers. No snapshot is injected or clock is paused.
    await Promise.all(pages.map(async (page, index) => {
      await expect.poll(() => latestSnapshot(games[index])!.state.bombs)
        .toEqual(expect.arrayContaining([expect.objectContaining({ id: old.id, range: 2 }), expect.objectContaining({ id: upgraded.id, range: 3 })]))
      await expectStats([page], ['0', '2', '3', '1'])
    }))
    await Promise.all(pages.map((page, index) =>
      page.screenshot({ path: testInfo.outputPath(`upgraded-bombs-${index}.png`), fullPage: true }),
    ))
    const collected = games[0].frames.flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
      .filter((event) => event.kind.type === 'pickupCollected')
    expect(collected.map((event) => event.kind)).toEqual(expect.arrayContaining([
      expect.objectContaining({ type: 'pickupCollected', kind: 'bombCapacity' }),
      expect.objectContaining({ type: 'pickupCollected', kind: 'blastRange' }),
    ]))
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
