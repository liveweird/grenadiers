import { selectClassicSettings } from '../tests/support/settings'
import { setTimeout as delay } from 'node:timers/promises'
import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'
import { observeBrowserPolicy } from '../tests/support/browser-policy'
import { startBrowserStallDiagnostics } from '../tests/support/browser-stall-diagnostics'
import { latestSnapshot, observeGame } from '../tests/support/duel'
import { expectVisibility, launchNaturalVisibilityBrowser } from '../tests/support/visibility'

test('native sound gesture creates Web Audio cues and mute prevents later playback', async ({ browserName }, testInfo) => {
  expect(browserName).toBe('chromium')
  const launched = await launchNaturalVisibilityBrowser()
  const diagnostics = startBrowserStallDiagnostics()
  try {
    const peerContext = await launched.browser.newContext({
      viewport: { width: 320, height: 844 }, reducedMotion: 'reduce',
    })
    const policies = await Promise.all([
      observeBrowserPolicy(launched.context), observeBrowserPolicy(peerContext),
    ])
    const one = launched.context.pages()[0]
    await one.setViewportSize({ width: 1440, height: 1050 })
    const two = await peerContext.newPage()
    await diagnostics.observePage(one, 'audio-primary')
    await diagnostics.observePage(two, 'audio-peer')
    const firstGame = observeGame(one)
    const secondGame = observeGame(two)
    const cdp = await launched.context.newCDPSession(one)
    const contexts: Array<{ contextId: string; contextState: string }> = []
    const nodes: Array<{ contextId: string; nodeType: string }> = []
    cdp.on('WebAudio.contextCreated', ({ context }) => contexts.push(context))
    cdp.on('WebAudio.contextChanged', ({ context }) => {
      const previous = contexts.find((candidate) => candidate.contextId === context.contextId)
      if (previous) previous.contextState = context.contextState
    })
    cdp.on('WebAudio.audioNodeCreated', ({ node }) => nodes.push(node))
    await cdp.send('WebAudio.enable')
    const baseUrl = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8085'
    await Promise.all([one, two].map((page) => page.goto(new URL('/', baseUrl).href)))
    expect(contexts).toEqual([])

    await expect(one.getByRole('button', { name: 'Enable sound', exact: true })).toBeVisible()
    expect((await new AxeBuilder({ page: one }).analyze()).violations).toEqual([])
    await one.screenshot({ path: testInfo.outputPath('sound-control-desktop.png'), fullPage: true })
    await two.getByText('Polski', { exact: true }).click()
    await expect(two.getByRole('button', { name: 'Włącz dźwięk', exact: true })).toBeVisible()
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    expect((await new AxeBuilder({ page: two }).analyze()).violations).toEqual([])
    await two.screenshot({ path: testInfo.outputPath('sound-control-polish-320.png'), fullPage: true })

    await one.getByRole('button', { name: 'Enable sound', exact: true }).click()
    await expect.poll(() => contexts[0]?.contextState).toBe('running')
    const contextId = contexts[0].contextId
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    await selectClassicSettings(one, firstGame)
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await one.getByRole('button', { name: 'Ready', exact: true }).click()
    await two.getByRole('button', { name: 'Gotowy', exact: true }).click()
    await Promise.all([firstGame, secondGame]
      .map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    const movementStart = firstGame.commands.length
    await one.keyboard.down('ArrowRight')
    await expect.poll(() => firstGame.commands.slice(movementStart)
      .some((command) => command.type === 'movement' && command.direction === 'east')).toBe(true)
    await one.getByRole('button', { name: 'Mute sound', exact: true }).click()
    await one.keyboard.up('ArrowRight')
    await expect.poll(() => firstGame.commands.slice(movementStart)
      .some((command) => command.type === 'movement' && command.direction === null)).toBe(true)
    expect(firstGame.commands.slice(movementStart).filter((command) => command.type === 'manualAction')).toEqual([])
    await one.getByRole('button', { name: 'Enable sound', exact: true }).click()
    await expect.poll(() => contexts.find((context) => context.contextId === contextId)?.contextState)
      .toBe('running')
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    const oscillatorCount = (): number => nodes.filter((node) => node.nodeType.includes('Oscillator')).length
    const beforeBomb = oscillatorCount()
    await one.keyboard.press('Space')
    await expect.poll(() => firstGame.commands.filter((command) => command.type === 'manualAction').length).toBe(1)
    await expect.poll(() => firstGame.frames.some((frame) => frame.type === 'snapshot'
      && frame.state.bombs.length === 1
      && frame.events.some((event) => event.kind.type === 'bombPlaced')), { intervals: [15] }).toBe(true)
    await expect.poll(oscillatorCount).toBeGreaterThan(beforeBomb)

    await one.getByRole('button', { name: 'Mute sound', exact: true }).click()
    await expect.poll(() => contexts.find((context) => context.contextId === contextId)?.contextState)
      .toBe('suspended')
    const mutedNodes = nodes.length
    await expect.poll(() => latestSnapshot(firstGame)?.state.bombs.length).toBe(0)
    await delay(250)
    expect(nodes).toHaveLength(mutedNodes)

    await one.getByRole('button', { name: 'Enable sound', exact: true }).click()
    await expect.poll(() => contexts.find((context) => context.contextId === contextId)?.contextState)
      .toBe('running')
    await delay(250)
    expect(nodes).toHaveLength(mutedNodes)
    const cover = await launched.context.newPage()
    await diagnostics.observePage(cover, 'visibility-cover')
    await cover.setContent('<title>Audio visibility cover</title>')
    await cover.bringToFront()
    await expectVisibility(one, 'hidden')
    await expect.poll(() => contexts.find((context) => context.contextId === contextId)?.contextState)
      .toBe('suspended')
    const hiddenNodes = nodes.length
    await one.bringToFront()
    await expectVisibility(one, 'visible')
    await expect(one.getByText('Sound paused', { exact: true })).toBeVisible()
    await one.getByRole('button', { name: 'Resume sound', exact: true }).click()
    await expect.poll(() => contexts.find((context) => context.contextId === contextId)?.contextState)
      .toBe('running')
    await delay(250)
    expect(nodes).toHaveLength(hiddenNodes)
    expect(policies.flatMap((policy) => policy.violations)).toEqual([])
  } finally {
    try { await launched.close() } finally {
      diagnostics.stop()
      await diagnostics.attach(testInfo, 'audio-browser-stall-diagnostics')
    }
  }
})
