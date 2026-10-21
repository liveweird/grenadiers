import { expect, test } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { arenaTilePixels } from '../tests/support/compositor'
import type { GameServerMessage } from '../../web/src/game/v11/schema'

test.beforeAll(() => expect(process.env.E2E_INVISIBILITY_CLOAK_FIXTURE)
  .toBe('arena-v2-invisibility-cloak'))

const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)

const collections = (game: ObservedGame) => snapshots(game).flatMap(({ events }) => events)
  .filter(({ kind }) => kind.type === 'pickupCollected'
    && kind.seat === 'one' && kind.kind === 'invisibilityCloak')

const cloakTicks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find(({ kind }) => kind === 'invisibilityCloak')?.remainingTicks ?? 0

test('two browsers observe recipient-private Cloak through reconnect and exact expiry', async ({ browser }, info) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two].map(installTransportProbe))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Invisibility cloak', exact: true }).check()
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await Promise.all([
      one.getByRole('button', { name: 'Ready', exact: true }).click(),
      two.getByRole('button', { name: 'Gotowy', exact: true }).click(),
    ])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak',
      'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
    ])
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await moveTo(one, games[0], { column: 2, row: 1 })
    await one.keyboard.press('Space')
    await moveTo(one, games[0], { column: 1, row: 1 })
    await moveTo(one, games[0], { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
      + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'invisibilityCloak',
    })
    await moveTo(one, games[0], { column: 1, row: 1 })
    await Promise.all([one, two].map(expectPaintedCanvas))
    const ownerFloor = await arenaTilePixels(one, 2, 1)
    const emptyFloor = await arenaTilePixels(two, 2, 1)
    await one.screenshot({ path: info.outputPath('cloak-before-collection-desktop-en.png'), fullPage: true })
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: info.outputPath('cloak-before-collection-polish-320-reduced.png'), fullPage: true })

    await moveTo(one, games[0], { column: 2, row: 1 })
    await moveTo(one, games[0], { column: 3, row: 1 })
    await expect.poll(() => cloakTicks(games[0])).toBeGreaterThan(0)
    expect(collections(games[0])).toHaveLength(1)
    await expect.poll(() => collections(games[1])).toHaveLength(0)
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.effects)
      .toContainEqual(expect.objectContaining({ kind: 'invisibilityCloak' }))
    const hidden = latestSnapshot(games[1])!.state.actors.find(({ seat }) => seat === 'one')
    expect(hidden).toEqual({ type: 'hidden', id: visibleActor(latestSnapshot(games[0]), 'one')!.id,
      seat: 'one', alive: true })
    await moveTo(one, games[0], { column: 2, row: 1 })
    expect(await arenaTilePixels(one, 2, 1)).not.toEqual(ownerFloor)
    expect(await arenaTilePixels(two, 2, 1)).toEqual(emptyFloor)

    const collectedAt = collections(games[0])[0].tick
    const deadline = collectedAt + 600
    await disconnectTransport(two)
    await expect.poll(async () => (await admissionBaseline(two))?.tick).toBeGreaterThan(collectedAt)
    await expect.poll(() => latestSnapshot(games[1])?.state.actors.find(({ seat }) => seat === 'one'))
      .toEqual(hidden)
    expect(collections(games[1])).toHaveLength(0)

    await expect.poll(() => latestSnapshot(games[1])?.state.physicalTick, { timeout: 10_000 })
      .toBeGreaterThanOrEqual(deadline)
    const retained = snapshots(games[0]).filter(({ state }) => state.physicalTick >= collectedAt)
    expect(retained.some(({ state }) => state.physicalTick < deadline)).toBe(true)
    for (const frame of retained) {
      const effect = visibleActor(frame, 'one')?.effects.find(({ kind }) => kind === 'invisibilityCloak')
      expect(effect?.remainingTicks ?? 0).toBe(Math.max(deadline - frame.state.physicalTick, 0))
    }
    await expect.poll(() => visibleActor(latestSnapshot(games[1]), 'one')?.position)
      .toEqual({ x: 2560, y: 1536 })
    expect(await arenaTilePixels(two, 2, 1)).not.toEqual(emptyFloor)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
