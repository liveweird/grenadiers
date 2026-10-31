import { expect, test } from '@playwright/test'
import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts'
import { latestSnapshot, observeGame, visibleActor } from '../tests/support/duel'
import { moveTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'

test('spike pits warn, permit escape, kill occupants and reset across reconnect and rematch',
  async ({ browser }, testInfo) => {
    expect(process.env.E2E_SPIKE_PITS_FIXTURE).toBe('arena-v3-spikes-seed2')
    const contexts = await createResponsiveBrowserContexts(browser)
    try {
      const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
      await installTransportProbe(two)
      const games = [observeGame(one), observeGame(two)]
      const room = () => games[0].frames.filter((frame) =>
        frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room
      await Promise.all([one.goto('/'), two.goto('/')])
      await one.getByRole('button', { name: 'Create room', exact: true }).click()
      await expect.poll(room).toBeTruthy()
      expect(room()!.settings.environment.spikePits).toEqual({
        enabled: true, warningTicks: 36, raisedTicks: 180,
      })
      await one.getByRole('button', { name: 'All off', exact: true }).click()
      await one.getByRole('checkbox', { name: 'Spike pits', exact: true }).check()
      await one.getByRole('textbox', { name: 'Warning time (seconds)', exact: true }).fill('2')
      await one.getByRole('textbox', { name: 'Raised time (seconds)', exact: true }).fill('1')
      await one.getByLabel('Round wins needed').selectOption('1')
      await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
      await expect.poll(() => room()?.settings.environment.spikePits).toEqual({
        enabled: true, warningTicks: 120, raisedTicks: 60,
      })
      await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(room()!.roomCode)
      await two.getByRole('button', { name: 'Join room', exact: true }).click()
      await Promise.all([one, two].map((page) =>
        page.getByRole('button', { name: 'Ready', exact: true }).click()))
      await Promise.all(games.map((game) =>
        expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
      const first = latestSnapshot(games[0])!
      expect(first.room.rulesVersion).toBe(10)
      expect(first.state.spikePits.map(({ cell }) => cell)).toContainEqual({ column: 5, row: 3 })
      expect(first.state.spikePits.every(({ phase }) => phase === 'idle')).toBe(true)
      await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
      for (const [column, row] of [[3, 1], [3, 3], [4, 3]]) {
        await moveTo(one, games[0], { column, row })
      }
      const pit = () => latestSnapshot(games[0])?.state.spikePits.find(({ cell }) =>
        cell.column === 5 && cell.row === 3)
      await moveTo(one, games[0], { column: 5, row: 3 })
      expect(pit()?.phase).toBe('warning')
      expect(pit()!.remainingTicks).toBeGreaterThan(0)
      await moveTo(one, games[0], { column: 4, row: 3 })
      await expect.poll(() => pit()?.phase, { intervals: [15] }).toBe('raised')
      expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(true)
      await one.screenshot({ path: testInfo.outputPath('spike-raised-desktop.png'), fullPage: true })
      await expect.poll(() => pit()?.phase, { intervals: [15] }).toBe('idle')

      const before = await admissionBaseline(two)
      await disconnectTransport(two)
      await expect.poll(async () => (await admissionBaseline(two))?.index ?? 0)
        .toBeGreaterThan(before!.index)
      expect((await admissionBaseline(two))?.stableCapability).toBe(true)
      await expect.poll(() => latestSnapshot(games[1])?.state.spikePits).toEqual(
        latestSnapshot(games[0])!.state.spikePits)
      await two.screenshot({ path: testInfo.outputPath('spike-idle-narrow-reduced.png'), fullPage: true })

      await moveTo(one, games[0], { column: 5, row: 3 })
      expect(pit()?.phase).toBe('warning')
      await expect.poll(() => room()?.phase.type).toBe('matchResult')
      expect(visibleActor(latestSnapshot(games[0]), 'one')?.alive).toBe(false)
      expect(pit()?.phase).toBe('raised')
      await Promise.all([one, two].map((page) =>
        page.getByRole('button', { name: 'Play again', exact: true }).click()))
      await expect.poll(() => room()?.phase.type).toBe('waiting')
      expect(room()!.settings.environment.spikePits).toEqual({
        enabled: true, warningTicks: 120, raisedTicks: 60,
      })
      await Promise.all([one, two].map((page) =>
        page.getByRole('button', { name: 'Ready', exact: true }).click()))
      await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('running')
      expect(latestSnapshot(games[0])!.state.spikePits.every(({ phase, remainingTicks }) =>
        phase === 'idle' && remainingTicks === 0)).toBe(true)
    } finally {
      await Promise.all(contexts.map((context) => context.close()))
    }
  })
