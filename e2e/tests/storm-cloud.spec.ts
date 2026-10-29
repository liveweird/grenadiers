import { expect, test, type Browser, type Page } from '@playwright/test'
import type { GameServerMessage, StormCloudView } from '../../web/src/game/v15/schema'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from './support/duel'
import { admissionBaseline, disconnectTransport, installTransportProbe } from './support/reconnect'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type Group = { pages: [Page, Page]; games: [ObservedGame, ObservedGame]; close: () => Promise<void> }

const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

function snapshots(game: ObservedGame): Snapshot[] {
  return game.frames.filter((frame): frame is Snapshot => frame.type === 'snapshot')
}

function sharedCloud(games: readonly ObservedGame[]): {
  tick: number
  clouds: [StormCloudView, StormCloudView]
} | undefined {
  for (const first of snapshots(games[0]!).toReversed()) {
    if (!first.state.stormCloud) continue
    const second = snapshots(games[1]!).findLast((frame) =>
      frame.state.physicalTick === first.state.physicalTick && frame.state.stormCloud !== null)
    if (second?.state.stormCloud) {
      return { tick: first.state.physicalTick, clouds: [first.state.stormCloud, second.state.stormCloud] }
    }
  }
  return undefined
}

async function createGroup(browser: Browser): Promise<Group> {
  const contexts = await Promise.all([
    browser.newContext(),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  const pages = await Promise.all(contexts.map((context) => context.newPage())) as [Page, Page]
  await installTransportProbe(pages[1])
  const games = pages.map(observeGame) as [ObservedGame, ObservedGame]
  await Promise.all(pages.map((page) => page.goto('/')))
  return {
    pages, games,
    close: async () => { await Promise.all(contexts.map((context) => context.close())) },
  }
}

async function joinRoom(group: Group): Promise<void> {
  const [one, two] = group.pages
  await one.getByRole('button', { name: 'Create room', exact: true }).click()
  await expect.poll(() => latestRoom(group.games[0])).toBeTruthy()
  const created = latestRoom(group.games[0])!
  expect(created.availableItems).toHaveLength(22)
  expect(created.settings.enabledItems).toEqual(created.availableItems)
  expect(created.settings.environment).toEqual({ stormCloud: true })
  expect(created.configuration.environment).toEqual({ stormCloud: true })
  await expect(one.getByRole('combobox', { name: 'Players', exact: true })).toHaveCount(0)
  await expect(one.getByRole('combobox', { name: 'Board size', exact: true })).toHaveCount(0)
  await expect(one.getByRole('combobox', { name: 'CPU difficulty', exact: true })).toHaveCount(0)
  await expect(one.getByRole('checkbox', { name: 'Storm cloud', exact: true })).toBeChecked()
  await two.getByText('Polski', { exact: true }).click()
  await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(created.roomCode)
  await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
  await expect.poll(() => latestRoom(group.games[1])?.roomId).toBe(created.roomId)
  await expect(two.getByRole('checkbox', { name: 'Chmura burzowa', exact: true })).toBeChecked()
}

async function startRound(group: Group): Promise<void> {
  await Promise.all([
    group.pages[0].getByRole('button', { name: 'Ready', exact: true }).click(),
    group.pages[1].getByRole('button', { name: 'Gotowy', exact: true }).click(),
  ])
  await Promise.all(group.games.map((game) =>
    expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
}

test('enabled storm cloud moves in one shared rendered world and survives reconnect and rematch',
  async ({ browser }, testInfo) => {
    test.setTimeout(90_000)
    const group = await createGroup(browser)
    try {
      await joinRoom(group)
      await group.pages[0].getByLabel('Round wins needed').selectOption('1')
      await group.pages[0].getByLabel('Bomb fuse').selectOption('60')
      await group.pages[0].getByRole('button', { name: 'Apply rules', exact: true }).click()
      await expect.poll(() => latestRoom(group.games[1])?.settings.winsToFinish).toBe(1)
      await startRound(group)

      await expect.poll(() => sharedCloud(group.games)).toBeTruthy()
      const baseline = sharedCloud(group.games)!
      expect(baseline.clouds[1]).toEqual(baseline.clouds[0])
      expect(Object.keys(baseline.clouds[0]).sort()).toEqual([
        'direction', 'legRemainingTicks', 'position',
      ])
      expect(baseline.clouds[0].legRemainingTicks).toBeGreaterThanOrEqual(0)
      expect(baseline.clouds[0].legRemainingTicks).toBeLessThanOrEqual(128)
      for (const game of group.games) {
        const frame = latestSnapshot(game)!
        expect(frame.room).toMatchObject({
          rulesVersion: 7, mapId: 'generated_v2',
          configuration: { environment: { stormCloud: true } },
          settings: { environment: { stormCloud: true } },
        })
        expect(frame.state.actors).toHaveLength(2)
        expect(frame.state.actors.every((actor) => actor.type !== 'visible'
          || actor.stormSlowRemainingTicks === 0)).toBe(true)
        expect(frame.state.stormCloud).not.toBeNull()
      }
      await expect.poll(() => {
        const current = sharedCloud(group.games)
        if (!current || current.tick <= baseline.tick) return false
        const before = baseline.clouds[0].position
        const after = current.clouds[0].position
        return before.x !== after.x || before.y !== after.y
      }).toBe(true)
      await Promise.all(group.pages.map(expectPaintedCanvas))
      await expect(group.pages[0].getByLabel('Phase time remaining')).toHaveText(/^\d+:\d{2}$/)
      await expect(group.pages[1].getByLabel('Czas pozostały w bieżącej fazie')).toHaveText(/^\d+:\d{2}$/)
      await Promise.all(group.pages.map((page) => expect(page.locator('.own-status-list')).toBeVisible()))
      await group.pages[0].screenshot({
        path: testInfo.outputPath('storm-cloud-running-desktop-en.png'), fullPage: true,
      })
      await group.pages[1].screenshot({
        path: testInfo.outputPath('storm-cloud-running-polish-320-reduced.png'), fullPage: true,
      })

      const admission = await admissionBaseline(group.pages[1])
      expect(admission).not.toBeNull()
      await disconnectTransport(group.pages[1])
      await expect.poll(async () => (await admissionBaseline(group.pages[1]))?.index ?? 0)
        .toBeGreaterThan(admission!.index)
      const restored = (await admissionBaseline(group.pages[1]))!
      expect(restored).toMatchObject({
        roomId: admission!.roomId, seat: 'two', hasState: true, stableCapability: true,
      })
      expect(restored.stormCloud).not.toBeNull()
      await expect.poll(() => snapshots(group.games[0]).find((frame) =>
        frame.state.physicalTick === restored.physicalTick)?.state.stormCloud).toEqual(restored.stormCloud)

      await group.pages[0].getByRole('region', { name: 'Grenadiers courtyard' }).click()
      await group.pages[0].keyboard.press('Space')
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('matchResult')))
      const completedRoundId = latestSnapshot(group.games[0])!.room.roundId
      await Promise.all([
        group.pages[0].getByRole('button', { name: 'Play again', exact: true }).click(),
        group.pages[1].getByRole('button', { name: 'Zagraj ponownie', exact: true }).click(),
      ])
      await Promise.all(group.games.map((game) =>
        expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')))
      for (const game of group.games) {
        expect(latestRoom(game)).toMatchObject({
          roundId: completedRoundId + 1,
          configuration: { environment: { stormCloud: true } },
          settings: { environment: { stormCloud: true } },
        })
      }
      await expect(group.pages[0].getByRole('checkbox', { name: 'Storm cloud', exact: true }))
        .toBeDisabled()
      await expect(group.pages[1].getByRole('checkbox', { name: 'Chmura burzowa', exact: true }))
        .toBeDisabled()
      await startRound(group)
      await Promise.all(group.games.map((game) => expect.poll(() => {
        const frame = latestSnapshot(game)
        return frame?.room.roundId === completedRoundId + 1
          && frame.state.actors.length === 2 && frame.state.stormCloud !== null
      }).toBe(true)))
    } finally {
      await group.close()
    }
  })
