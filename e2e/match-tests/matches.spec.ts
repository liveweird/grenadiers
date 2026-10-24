import { selectClassicSettings } from '../tests/support/settings'
import AxeBuilder from '@axe-core/playwright'
import { expect, test, type Page } from '@playwright/test'
import type { RoomView } from '../../web/src/game/v10/schema'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'

test.beforeAll(() => {
  expect(process.env.E2E_MATCH_FIXTURE, 'Configure the documented generated match fixture before acceptance')
    .toBe('arena-v1-matches')
})

function latestRoom(game: ObservedGame): RoomView | undefined {
  const frame = game.frames.filter((value) => value.type === 'roomState' || value.type === 'snapshot').at(-1)
  return frame?.room
}

async function running(games: ObservedGame[], roundNumber: number) {
  await Promise.all(games.map((game) => expect.poll(() => {
    const room = latestSnapshot(game)?.room
    return room?.phase.type === 'running' ? room.roundNumber : 0
  }).toBe(roundNumber)))
  for (const game of games) {
    const frame = latestSnapshot(game)!
    expect(frame.room).toMatchObject({ rulesVersion: 5, mapId: 'generated_v2' })
    expect(frame.state.bombs).toEqual([])
    expect(frame.state.flames).toEqual([])
    expect(frame.state.pickups).toEqual([])
    expect(frame.state.remainingTicks).toBeGreaterThan(10_600)
    for (const actor of frame.state.actors) {
      expect(actor).toMatchObject({ type: 'visible', alive: true, bombCapacity: 1, bombRange: 2, speedLevel: 0, liveBombCount: 0 })
      if (actor.type !== 'visible') throw new Error('Classic fixture actors must remain visible')
      const northwest = (actor.seat === 'one') === (roundNumber % 2 === 1)
      expect(actor.position).toEqual(northwest ? { x: 1536, y: 1536 } : { x: 13_824, y: 11_776 })
    }
  }
}

async function plant(page: Page, game: ObservedGame) {
  await expect(page.locator('.game-arena canvas')).toBeVisible()
  await page.getByRole('region', { name: 'Grenadiers courtyard', exact: true }).click()
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)?.state.bombs.length, { intervals: [15] }).toBe(1)
}

async function expectActorStatLabelsContained(page: Page, expectedColumnCount: number) {
  const geometry = await page.locator('.actor-stats').evaluateAll((grids) => grids.map((grid) => {
    const cells = [...grid.querySelectorAll<HTMLElement>(':scope > div')]
    const labels = cells.map((cell, index) => {
      const range = document.createRange()
      range.selectNodeContents(cell.querySelector('dt')!)
      return { cell: cell.getBoundingClientRect(), index, text: [...range.getClientRects()] }
    })
    const overflow = labels.flatMap(({ cell, index, text }) => text
      .filter(({ left, right }) => left < cell.left - 0.5 || right > cell.right + 0.5)
      .map(() => index))
    const overlaps: string[] = []
    for (const [index, label] of labels.entries()) {
      for (const other of labels.slice(index + 1)) {
        if (label.text.some((first) => other.text.some((second) =>
          first.left < second.right - 0.5 && first.right > second.left + 0.5
          && first.top < second.bottom - 0.5 && first.bottom > second.top + 0.5))) {
          overlaps.push(`${label.index}:${other.index}`)
        }
      }
    }
    const columnCount = new Set(labels.map(({ cell }) => Math.round(cell.left * 2) / 2)).size
    return { cellCount: cells.length, columnCount, fragmentCounts: labels.map(({ text }) => text.length), overflow, overlaps }
  }))
  expect(geometry).toHaveLength(2)
  for (const grid of geometry) {
    expect(grid).toMatchObject({ cellCount: 8, columnCount: expectedColumnCount, overflow: [], overlaps: [] })
    expect(grid.fragmentCounts).toHaveLength(8)
    expect(grid.fragmentCounts.every((count) => count > 0)).toBe(true)
  }
}

async function finished(games: ObservedGame[], roundNumber: number, final: boolean) {
  const expectedPhase = final ? 'matchResult' : 'result'
  await Promise.all(games.map((game) => expect.poll(() => {
    const room = latestSnapshot(game)?.room
    return room?.phase.type === expectedPhase ? room.lastRound?.roundNumber : 0
  }).toBe(roundNumber)))
  const rooms = games.map((game) => latestSnapshot(game)!.room)
  expect(rooms[1].lastRound).toEqual(rooms[0].lastRound)
  expect(rooms[1].scores).toEqual(rooms[0].scores)
  return rooms
}

test('two browsers finish a drawn round, first-to-three match, and consensual rematch', async ({ browser }, testInfo) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  const games: ObservedGame[] = []
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    const [one, two] = pages
    games.push(...pages.map(observeGame))
    await Promise.all(pages.map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    await selectClassicSettings(one, games[0])
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await running(games, 1)
    await Promise.all(pages.map(expectPaintedCanvas))
    await one.getByRole('region', { name: 'Grenadiers courtyard', exact: true }).click()

    // The public fixture has an ordinary open corridor. One shared bomb gives
    // simultaneous elimination without synchronizing two independent commands.
    for (const [column, row] of [[5, 1], [5, 5], [7, 5], [7, 7], [9, 7], [9, 9], [11, 9], [11, 11], [13, 11]]) {
      await moveTo(one, games[0], { column, row })
    }
    await plant(one, games[0])
    const drawn = await finished(games, 1, false)
    expect(drawn[0].lastRound?.result).toEqual({ type: 'draw', reason: 'mutualElimination' })
    expect(drawn[0].scores).toEqual({ one: 0, two: 0 })
    for (const game of games) expect(latestSnapshot(game)!.state.actors.every((actor) => !actor.alive)).toBe(true)
    await Promise.all(pages.map((page, index) =>
      page.screenshot({ path: testInfo.outputPath(`draw-${index}.png`), fullPage: true }),
    ))

    let previousEpoch = drawn[0].roundId
    for (const roundNumber of [2, 3, 4]) {
      await running(games, roundNumber)
      const started = latestSnapshot(games[0])!.room
      expect(started.roundId).toBeGreaterThan(previousEpoch)
      expect(games[0].frames.some((frame) => frame.type === 'roomState'
        && frame.room.phase.type === 'countdown' && frame.room.phase.kind === 'interRound'
        && frame.room.roundId === started.roundId && frame.room.roundNumber === roundNumber - 1)).toBe(true)
      await plant(one, games[0])
      const rooms = await finished(games, roundNumber, roundNumber === 4)
      expect(rooms[0].lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
      expect(rooms[0].scores).toEqual({ one: 0, two: roundNumber - 1 })
      previousEpoch = started.roundId
    }

    const final = latestRoom(games[0])!
    expect(final.phase).toMatchObject({ type: 'matchResult', result: { type: 'winner', seat: 'two', reason: 'roundWins' } })
    await expect(one.getByRole('button', { name: 'Play again', exact: true })).not.toBeVisible()
    await Promise.all(pages.map((page) =>
      expect(page.getByRole('status')).toHaveText('Player two wins the match with 3 round wins.'),
    ))
    await Promise.all(pages.map((page) => expect(page.getByRole('button', { name: 'Play again', exact: true })).toBeVisible()))
    await Promise.all(pages.map(async (page, index) => {
      expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
      expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
      await expect(page.getByLabel('Round wins for Player two')).toContainText('3')
      if (index === 0) await expectActorStatLabelsContained(page, 4)
      await page.screenshot({ path: testInfo.outputPath(`match-result-${index}.png`), fullPage: true })
    }))

    await two.getByText('Polski', { exact: true }).click()
    await expect(two.getByRole('status')).toHaveText('Gracz drugi wygrywa mecz (wygrane rundy: 3).')
    await expect(two.getByLabel('Wygrane rundy: Gracz drugi')).toContainText('3')
    await expectActorStatLabelsContained(two, 2)
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    expect((await new AxeBuilder({ page: two }).analyze()).violations).toEqual([])
    await two.screenshot({ path: testInfo.outputPath('match-result-polish-mobile.png'), fullPage: true })
    await two.getByText('English', { exact: true }).click()

    await one.getByRole('button', { name: 'Play again', exact: true }).click()
    await expect.poll(() => latestRoom(games[1])?.seats[0].rematchReady).toBe(true)
    await one.getByRole('button', { name: 'Withdraw rematch vote', exact: true }).click()
    await expect.poll(() => latestRoom(games[1])?.seats[0].rematchReady).toBe(false)
    expect(latestRoom(games[1])?.phase.type).toBe('matchResult')
    await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Play again', exact: true }).click()))
    await Promise.all(games.map((game) => expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')))
    for (const [index, page] of pages.entries()) {
      const reset = latestRoom(games[index])!
      expect(reset).toMatchObject({ roundId: previousEpoch + 1, roundNumber: 0, scores: { one: 0, two: 0 }, lastRound: null })
      expect(reset.tick).toBeGreaterThan(final.tick)
      expect(reset.seats.every((seat) => seat.occupied && !seat.ready && !seat.rematchReady)).toBe(true)
      await expect(page.locator('.game-arena canvas')).not.toBeVisible()
      await expect(page.getByRole('button', { name: 'Ready', exact: true })).toBeVisible()
    }
    await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
    await running(games, 1)
    await plant(one, games[0])
    expect(games[0].commands.filter((command) => command.type === 'manualAction').map((command) => command.actionId))
      .toEqual([1, 1, 1, 1, 1])
    expect(games.map((game) => game.sockets)).toEqual([1, 1])
    for (const page of pages) await expect(page.getByRole('alert')).not.toBeVisible()
  } catch (error) {
    await testInfo.attach('match-public-history', {
      contentType: 'application/json',
      body: JSON.stringify(games.map((game) => ({
        commands: game.commands.slice(-128),
        errors: game.frames.filter((frame) => frame.type === 'error').slice(-32),
        rooms: game.frames.filter((frame) => frame.type === 'snapshot' || frame.type === 'roomState')
          .slice(-120).map((frame) => ({
            revision: frame.room.revision, tick: frame.room.tick, roundId: frame.room.roundId,
            roundNumber: frame.room.roundNumber, phase: frame.room.phase, scores: frame.room.scores,
            lastRound: frame.room.lastRound,
            ...(frame.type === 'snapshot' ? { actors: frame.state.actors, bombs: frame.state.bombs } : {}),
          })),
      }))),
    })
    throw error
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
