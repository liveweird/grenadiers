import { selectClassicSettings } from '../tests/support/settings'
import { setTimeout as delay } from 'node:timers/promises'
import { expect, test, type BrowserContext, type Page, type TestInfo } from '@playwright/test'
import { latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'
import {
  expectStableFreshPosition, expectVisibility, launchNaturalVisibilityBrowser,
  observeVisibilityChanges, position, visibilityChanges,
} from '../tests/support/visibility'
import { publicErrorSummary, startupDiagnosticsFrom } from '../tests/support/visibility-startup'

test.beforeAll(() => {
  expect(process.env.E2E_MATCH_FIXTURE, 'Configure the documented match fixture for hardening acceptance')
    .toBe('arena-v1-matches')
})

async function joinRunning(pages: Page[], games: ObservedGame[]): Promise<void> {
  const [one, two] = pages
  const baseUrl = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8085'
  await Promise.all(pages.map((page) => page.goto(new URL('/', baseUrl).href)))
  await one.getByRole('button', { name: 'Create room', exact: true }).click()
  await selectClassicSettings(one, games[0])
  const code = one.getByLabel('Room invitation code', { exact: true })
  await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
  await two.getByRole('button', { name: 'Join room', exact: true }).click()
  await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
  await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
}

async function attachPublicSummary(testInfo: TestInfo, games: ObservedGame[]): Promise<void> {
  await testInfo.attach('visibility-public-summary', {
    contentType: 'application/json',
    body: JSON.stringify(games.map((game) => ({
      sockets: game.sockets,
      commands: game.commands.slice(-64),
      snapshots: game.frames.filter((frame) => frame.type === 'snapshot').slice(-24)
        .map((frame) => ({ room: frame.room, actors: frame.state.actors })),
    }))),
  })
}

async function attachStartupSummary(testInfo: TestInfo, error: unknown,
  launched?: Awaited<ReturnType<typeof launchNaturalVisibilityBrowser>>): Promise<void> {
  const diagnostics = startupDiagnosticsFrom(error) ?? launched?.startupDiagnostics()
  if (!diagnostics) return
  await testInfo.attach('visibility-chromium-startup', {
    contentType: 'application/json', body: JSON.stringify(diagnostics),
  })
}

test('a hidden gameplay tab clears held input and recovers on a fresh native action', async ({ browserName }, testInfo) => {
  expect(browserName).toBe('chromium')
  let launched: Awaited<ReturnType<typeof launchNaturalVisibilityBrowser>> | undefined
  let peerContext: BrowserContext | undefined
  const games: ObservedGame[] = []
  let failure: unknown
  try {
    launched = await launchNaturalVisibilityBrowser()
    peerContext = await launched.browser.newContext()
    const one = launched.context.pages()[0]
    const two = await peerContext.newPage()
    const pages = [one, two]
    games.push(...pages.map(observeGame))
    await joinRunning(pages, games)
    const cover = await launched.context.newPage()
    await cover.setContent('<title>Visibility cover</title><main>Cover tab</main>')
    await one.bringToFront()
    await expectVisibility(one, 'visible')
    await observeVisibilityChanges(one)
    const arena = one.getByRole('region', { name: 'Grenadiers courtyard', exact: true })
    await arena.click()

    const socketCount = games[0].sockets
    const commandStart = games[0].commands.length
    const initial = position(games[1], 'one')
    await one.keyboard.down('ArrowRight')
    await expect.poll(() => games[0].commands.slice(commandStart)
      .some((command) => command.type === 'movement' && command.direction === 'east')).toBe(true)
    await expect.poll(() => position(games[1], 'one').x, { intervals: [15] }).toBeGreaterThan(initial.x)

    const beforeHide = latestSnapshot(games[1])!
    const hiddenAt = performance.now()
    await cover.bringToFront()
    await expectVisibility(one, 'hidden')
    await expect.poll(() => visibilityChanges(one)).toContain('hidden')
    await expect.poll(() => games[0].commands.slice(commandStart)
      .some((command) => command.type === 'movement' && command.direction === null)).toBe(true)
    const stopped = await expectStableFreshPosition(games[1], 'one', beforeHide.room.revision)
    await delay(1_050)
    expect(performance.now() - hiddenAt).toBeGreaterThanOrEqual(1_000)
    await expect.poll(() => latestSnapshot(games[1])!.room.tick, { intervals: [15] })
      .toBeGreaterThan(beforeHide.room.tick + 50)
    const hiddenStable = await expectStableFreshPosition(games[1], 'one', stopped.revision)
    expect(hiddenStable.position).toEqual(stopped.position)
    expect(games[0].sockets).toBe(socketCount)

    await one.bringToFront()
    await expectVisibility(one, 'visible')
    await expect.poll(() => visibilityChanges(one)).toEqual(['hidden', 'visible'])
    await one.keyboard.up('ArrowRight')
    const returned = await expectStableFreshPosition(games[1], 'one', hiddenStable.revision)
    expect(returned.position).toEqual(stopped.position)
    expect(games[0].commands.slice(commandStart)
      .filter((command) => command.type === 'manualAction')).toEqual([])

    await arena.click()
    const freshStart = games[0].commands.length
    await one.keyboard.down('ArrowLeft')
    try {
      await expect.poll(() => games[0].commands.slice(freshStart)
        .some((command) => command.type === 'movement' && command.direction === 'west')).toBe(true)
      await expect.poll(() => position(games[1], 'one').x, { intervals: [15] })
        .toBeLessThan(returned.position.x)
    } finally {
      await one.keyboard.up('ArrowLeft')
    }
    await expect.poll(() => games[0].commands.slice(freshStart)
      .some((command) => command.type === 'movement' && command.direction === null)).toBe(true)
    await expectStableFreshPosition(games[1], 'one', returned.revision)
    await expect(two.getByLabel('Phase time remaining')).toBeVisible()
    await expect(two.getByText('Alive', { exact: true })).toHaveCount(2)
    expect(games[0].sockets).toBe(socketCount)
  } catch (error) {
    failure = error
    try {
      await attachPublicSummary(testInfo, games)
    } catch (attachmentError) {
      failure = new AggregateError([error, attachmentError],
        'Visibility scenario and public-summary attachment both failed', { cause: error })
    }
  }
  if (launched) {
    try {
      await launched.close()
    } catch (cleanupError) {
      failure = failure === undefined ? cleanupError : new AggregateError(
        [failure, cleanupError],
        `Visibility scenario failed (${publicErrorSummary(failure)}); browser cleanup failed `
        + `(${publicErrorSummary(cleanupError)})`,
        { cause: failure instanceof Error ? failure : undefined })
    }
  }
  if (failure !== undefined) {
    try {
      await attachStartupSummary(testInfo, failure, launched)
    } catch (attachmentError) {
      failure = new AggregateError([failure, attachmentError],
        'Visibility failure and startup-summary attachment both failed', { cause: failure })
    }
    throw failure
  }
})
