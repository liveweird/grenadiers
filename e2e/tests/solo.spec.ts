import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'
import { latestSnapshot, observeGame, visibleActor, type ObservedGame } from './support/duel'
import { admissionBaseline, disconnectTransport, installTransportProbe } from './support/reconnect'
import { gameFrames, openGame, sendGame } from './support/game'

const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

test('a solo player completes, rematches, reconnects and leaves a real CPU match', async ({ browser }) => {
  test.setTimeout(60_000)
  const contexts = await Promise.all([browser.newContext(), browser.newContext()])
  try {
    const [player, authority] = await Promise.all(contexts.map((context) => context.newPage()))
    await installTransportProbe(player)
    const game = observeGame(player)
    await Promise.all([player.goto('/'), openGame(authority)])

    await player.getByRole('button', { name: 'Play solo against CPU', exact: true }).click()
    await expect.poll(() => latestRoom(game)).toBeTruthy()
    const created = latestRoom(game)!
    expect(created.mode).toEqual({ type: 'solo', difficulty: 'standard' })
    expect(created.seats).toEqual([
      expect.objectContaining({
        seat: 'one', occupied: true, controller: { type: 'human' }, connection: { type: 'connected' },
      }),
      expect.objectContaining({
        seat: 'two', occupied: true, controller: { type: 'cpu', difficulty: 'standard' },
        connection: { type: 'serverControlled' },
      }),
    ])
    expect(created.settings.enabledItems).toEqual(created.availableItems)
    await expect(player.getByText('Private solo match · Standard CPU', { exact: true })).toBeVisible()
    await expect(player.getByRole('list', { name: 'Players' })).toContainText('CPU opponent')
    await expect(player.getByLabel('Room invitation code', { exact: true })).toHaveCount(0)

    const joinSequence = await sendGame(authority, { type: 'joinRoom', roomCode: created.roomCode })
    await expect.poll(() => gameFrames(authority)).toContainEqual({
      type: 'error', code: 'roomFull', sequence: joinSequence,
    })
    const resumeSequence = await sendGame(authority, {
      type: 'resumeRoom', roomId: created.roomId, resumeToken: '0'.repeat(64),
    })
    await expect.poll(() => gameFrames(authority)).toContainEqual({
      type: 'error', code: 'resumeRejected', sequence: resumeSequence,
    })

    await player.getByLabel('Round wins needed').selectOption('1')
    await player.getByLabel('Bomb fuse').selectOption('60')
    await player.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await expect.poll(() => latestRoom(game)?.settings.winsToFinish).toBe(1)
    expect(latestRoom(game)?.settings).toMatchObject({
      bombFuseTicks: 60, enabledItems: created.availableItems,
    })

    await expect.poll(() => latestRoom(game)?.seats[1]?.ready).toBe(true)
    await player.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
    const firstCpuPosition = { ...visibleActor(latestSnapshot(game), 'two')!.position }
    await expect.poll(() => game.frames.filter((frame) => frame.type === 'snapshot').some((frame) => {
      const cpu = visibleActor(frame, 'two')
      return Boolean(cpu && (cpu.position.x !== firstCpuPosition.x || cpu.position.y !== firstCpuPosition.y))
    })).toBe(true)

    const humanId = visibleActor(latestSnapshot(game), 'one')!.id
    await player.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await player.keyboard.press('Space')
    await expect.poll(() => game.commands.some((command) =>
      command.type === 'manualAction' && command.action === 'bomb')).toBe(true)
    await expect.poll(() => game.frames.filter((frame) => frame.type === 'snapshot')
      .some((frame) => frame.state.bombs.some((bomb) => bomb.owner === humanId)),
    { intervals: [15] }).toBe(true)
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('matchResult')
    const result = latestSnapshot(game)!
    expect(visibleActor(result, 'one')?.alive).toBe(false)
    expect(result.room.phase).toMatchObject({
      type: 'matchResult', result: { type: 'winner', seat: 'two', reason: 'roundWins' },
    })
    expect(result.room.scores).toEqual({ one: 0, two: 1 })
    await expect(player.getByRole('status')).toHaveText('CPU opponent wins the match with one round win.')

    const rematch = player.getByRole('button', { name: 'Play again', exact: true })
    await expect(rematch).toBeEnabled({ timeout: 10_000 })
    await rematch.click()
    await expect.poll(() => latestRoom(game)?.phase.type).toBe('waiting')
    expect(latestRoom(game)?.settings).toMatchObject({
      winsToFinish: 1, bombFuseTicks: 60, enabledItems: created.availableItems,
    })

    await expect.poll(() => latestRoom(game)?.seats[1]?.ready).toBe(true)
    await player.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')

    const admission = await admissionBaseline(player)
    expect(admission).not.toBeNull()
    await disconnectTransport(player)
    await expect.poll(async () => (await admissionBaseline(player))?.index).toBeGreaterThan(admission!.index)
    const restored = (await admissionBaseline(player))!
    expect(restored).toMatchObject({ roomId: created.roomId, seat: 'one', stableCapability: true })
    expect(latestRoom(game)?.seats[1]).toMatchObject({
      controller: { type: 'cpu', difficulty: 'standard' }, connection: { type: 'serverControlled' },
    })

    await player.getByRole('button', { name: 'Leave room', exact: true }).click()
    await expect(player.getByRole('button', { name: 'Play solo against CPU', exact: true })).toBeEnabled()
    const staleJoin = await sendGame(authority, { type: 'joinRoom', roomCode: created.roomCode })
    await expect.poll(() => gameFrames(authority)).toContainEqual({
      type: 'error', code: 'roomNotFound', sequence: staleJoin,
    })
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})

test('Polish solo controls stay accessible without horizontal overflow at 320 pixels', async ({ page }) => {
  await page.setViewportSize({ width: 320, height: 844 })
  await page.goto('/')
  await page.getByText('Polski', { exact: true }).click()
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  await page.getByRole('button', { name: 'Zagraj solo z komputerem', exact: true }).click()

  await expect(page.getByText('Prywatny mecz solo · Standardowy komputer', { exact: true })).toBeVisible()
  await expect(page.getByRole('list', { name: 'Gracze' })).toContainText('Komputer')
  await expect(page.getByRole('button', { name: 'Gotowy', exact: true })).toBeVisible()
  await expect(page.getByLabel('Kod zaproszenia do pokoju', { exact: true })).toHaveCount(0)
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  expect((await new AxeBuilder({ page }).analyze()).violations).toEqual([])
})
