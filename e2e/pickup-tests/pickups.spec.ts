import { expect, test, type Page } from '@playwright/test'
import { moveTo } from '../tests/support/movement'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'

test.beforeAll(() => {
  expect(process.env.E2E_PICKUP_FIXTURE, 'Configure the documented server seed fixture before pickup acceptance')
    .toBe('arena-v1-pickups')
})

function actor(game: ObservedGame) {
  const value = latestSnapshot(game)?.state.actors.find((candidate) => candidate.seat === 'one')
  if (!value?.alive) throw new Error('Fixture route requires a living first player')
  return value
}

async function plant(page: Page, game: ObservedGame) {
  const previous = latestSnapshot(game)!.state.bombs.map((bomb) => bomb.id)
  await page.keyboard.press('Space')
  await expect.poll(() => latestSnapshot(game)!.state.bombs.some((bomb) => !previous.includes(bomb.id)), { intervals: [15] }).toBe(true)
  return latestSnapshot(game)!.state.bombs.find((bomb) => !previous.includes(bomb.id))!
}

async function clearHazards(game: ObservedGame) {
  await expect.poll(() => {
    const state = latestSnapshot(game)!.state
    return state.bombs.length + state.flames.length
  }, { intervals: [30] }).toBe(0)
}

async function escapeBomb(
  page: Page,
  game: ObservedGame,
  legs: readonly {
    key: 'ArrowLeft' | 'ArrowRight' | 'ArrowUp' | 'ArrowDown'
    direction: 'west' | 'east' | 'north' | 'south'
    safelyClear: () => boolean
    centerTurn?: Cell
  }[],
  destination: Cell,
) {
  for (const leg of legs) {
    const commandCount = game.commands.length
    await page.keyboard.down(leg.key)
    try {
      await expect.poll(() => game.commands.slice(commandCount)
        .some((command) => command.type === 'movement' && command.direction === leg.direction))
        .toBe(true)
      await expect.poll(() => leg.safelyClear(), { intervals: [15] }).toBe(true)
    } finally {
      await page.keyboard.up(leg.key)
    }
    await expect.poll(() => {
      const commands = game.commands.slice(commandCount)
      const movement = commands.findIndex((command) => command.type === 'movement'
        && command.direction === leg.direction)
      return movement >= 0 && commands.slice(movement + 1)
        .some((command) => command.type === 'movement' && command.direction === null)
    }, { intervals: [15] }).toBe(true)
    // A slow key-up can reach the outer wall (192 units from center). The
    // simulation permits perpendicular turns only within 128, so correct that
    // bounded corner offset before turning, rather than centering a whole cell.
    if (leg.centerTurn) {
      const releasedRevision = latestSnapshot(game)!.room.revision
      await expect.poll(() => {
        const frames = game.frames.filter((frame) => frame.type === 'snapshot').slice(-2)
        if (frames.length !== 2 || frames[0].room.revision <= releasedRevision) return false
        const positions = frames.map((frame) => frame.state.actors.find((value) => value.seat === 'one')!.position)
        return positions.every((value) => value.x === positions[0].x && value.y === positions[0].y)
      }, { intervals: [15] }).toBe(true)
      await moveTo(page, game, leg.centerTurn)
    }
  }
  // Once the actor is outside the blast footprint, let the real fuse and flames
  // finish before spending time on exact fixture-center assertions.
  await clearHazards(game)
  await moveTo(page, game, destination)
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
  const games: ObservedGame[] = []
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    const [one, two] = pages
    games.push(...pages.map(observeGame))
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
    await escapeBomb(one, games[0], [{
      key: 'ArrowLeft',
      direction: 'west',
      safelyClear: () => actor(games[0]).position.x <= 1_664,
      centerTurn: { column: 1, row: 1 },
    }, {
      key: 'ArrowDown',
      direction: 'south',
      safelyClear: () => actor(games[0]).position.y >= 2_560,
    }], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups)
      .toContainEqual({ cell: { column: 3, row: 1 }, kind: 'bombCapacity' })
    await one.screenshot({ path: testInfo.outputPath('capacity-revealed.png'), fullPage: true })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await expectStats(pages, ['2', '2', '2', '1'])

    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await plant(one, games[0])
    await escapeBomb(one, games[0], [{
      key: 'ArrowUp',
      direction: 'north',
      safelyClear: () => actor(games[0]).position.y <= 1_664,
      centerTurn: { column: 1, row: 1 },
    }, {
      key: 'ArrowRight',
      direction: 'east',
      safelyClear: () => actor(games[0]).position.x >= 2_944,
    }], { column: 3, row: 1 })
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
  } catch (error) {
    // Trace DOM snapshots are disabled for real-time input. Preserve bounded,
    // redacted transport evidence after failure, without authentication frames.
    await testInfo.attach('pickup-websocket-summary', {
      contentType: 'application/json',
      body: JSON.stringify(games.map((game) => ({
        sockets: game.sockets,
        commands: game.commands.slice(-128),
        snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-120)
          .map((frame) => ({
            revision: frame.room.revision,
            roundId: frame.room.roundId,
            phase: frame.room.phase,
            tick: frame.room.tick,
            actors: frame.state.actors,
            bombs: frame.state.bombs,
            pickups: frame.state.pickups,
            flameCount: frame.state.flames.length,
            events: frame.events,
          })),
      }))),
    })
    throw error
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
