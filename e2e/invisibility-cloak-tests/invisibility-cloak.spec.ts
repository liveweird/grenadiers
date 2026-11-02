import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { expect, test } from '@playwright/test'
import { arenaTilePixels } from '../tests/support/compositor'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import { applyItemSettings } from '../tests/support/settings'
import type { GameServerMessage } from '../../web/src/game/v20/schema'

test.beforeAll(() => expect(process.env.E2E_INVISIBILITY_CLOAK_FIXTURE)
  .toBe('arena-v3-invisibility-cloak'))

const snapshots = (game: ObservedGame) => game.frames.filter(
  (frame): frame is Extract<GameServerMessage, { type: 'snapshot' }> => frame.type === 'snapshot',
)

const collections = (game: ObservedGame) => snapshots(game).flatMap(({ events }) => events)
  .filter(({ kind }) => kind.type === 'pickupCollected'
    && kind.seat === 'one' && kind.kind === 'invisibilityCloak')

const cloakTicks = (game: ObservedGame) => visibleActor(latestSnapshot(game), 'one')?.effects
  .find(({ kind }) => kind === 'invisibilityCloak')?.remainingTicks ?? 0

test('two browsers observe recipient-private Cloak through reconnect and exact expiry', async ({ browser }, info) => {
  const contexts = await createResponsiveBrowserContexts(browser)
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two].map(installTransportProbe))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await applyItemSettings(one, games[0], [{ id: 'invisibilityCloak', label: 'Invisibility cloak' }])
    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await Promise.all([
      one.getByRole('button', { name: 'Ready', exact: true }).click(),
      two.getByRole('button', { name: 'Gotowy', exact: true }).click(),
    ])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    expect(latestSnapshot(games[0])!.room.availableItems).toEqual([
      'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak',
      'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter',
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
    const settledOwner = visibleActor(latestSnapshot(games[0]), 'one')
    if (!settledOwner) throw new Error('Cloak owner must remain visible to its own client')
    const settledOwnerPosition = { ...settledOwner.position }
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
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.position).toEqual(settledOwnerPosition)
    await expect.poll(() => visibleActor(latestSnapshot(games[1]), 'one')?.position)
      .toEqual(settledOwnerPosition)
    expect(await arenaTilePixels(two, 2, 1)).not.toEqual(emptyFloor)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
