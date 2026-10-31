import { expect, test, type Page, type TestInfo } from '@playwright/test'
import { latestSnapshot, observeGame } from './support/duel'
import { selectClassicSettings } from './support/settings'
import { observeBrowserPolicy } from './support/browser-policy'

declare global {
  interface Window { roundCameraTestClock: { freeze(): void; advance(ms: number): void } }
}

async function courtyardCenter(page: Page, info: TestInfo, name: string): Promise<Buffer> {
  await page.locator('.game-arena canvas').scrollIntoViewIfNeeded()
  const bounds = await page.locator('.game-arena canvas').boundingBox()
  if (!bounds) throw new Error('Arena canvas unavailable')
  // Static center tiles exclude both unmoving players, which spawn in opposite corners.
  return page.screenshot({ path: info.outputPath(`${name}.png`), clip: {
    x: bounds.x + bounds.width * .3, y: bounds.y + bounds.height * .3,
    width: bounds.width * .4, height: bounds.height * .4,
  } })
}

test('real snapshot-free countdowns produce camera intros on the first and subsequent rounds', async ({ browser }, info) => {
  test.setTimeout(60_000)
  const contexts = await Promise.all([0, 1].map(() => browser.newContext({ viewport: { width: 1440, height: 1050 } })))
  const context = contexts[0]
  try {
    const policy = await observeBrowserPolicy(context)
    const one = await context.newPage()
    await one.addInitScript(() => {
      const now = performance.now.bind(performance)
      const raf = window.requestAnimationFrame.bind(window)
      let frozenAt: number | null = null
      Object.defineProperty(performance, 'now', { configurable: true, value: () => frozenAt ?? now() })
      window.requestAnimationFrame = (callback) => raf((timestamp) => callback(frozenAt ?? timestamp))
      window.roundCameraTestClock = {
        freeze: () => { frozenAt = now() },
        advance: (ms) => { if (frozenAt !== null) frozenAt += ms },
      }
    })
    const two = await contexts[1].newPage()
    const game = observeGame(one)
    const peer = observeGame(two)
    const errors: string[] = []
    for (const page of [one, two]) page.on('pageerror', (error) => errors.push(error.message))
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    await selectClassicSettings(one, game)
    const invitation = await one.getByLabel('Room invitation code', { exact: true }).innerText()
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(invitation)
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await expect.poll(() => peer.frames.some((frame) => frame.type === 'admitted')).toBe(true)
    await one.evaluate(() => window.roundCameraTestClock.freeze())
    await one.getByRole('button', { name: 'Ready', exact: true }).click()
    await two.getByRole('button', { name: 'Ready', exact: true }).click()

    try {
      for (const index of [1, 2]) {
        await expect.poll(() => {
          const snapshot = latestSnapshot(game)
          return snapshot ? `${snapshot.room.roundNumber}:${snapshot.room.phase.type}` : 'none'
        }, { intervals: [20], timeout: 20_000 }).toBe(`${index}:running`)
        const currentRound = latestSnapshot(game)!.room.roundId
        await expect(one.locator('.game-arena canvas')).toBeVisible()
        const opening = await courtyardCenter(one, info, `live-round-${index}-opening`)
        await one.evaluate(() => window.roundCameraTestClock.advance(1_000))
        await one.evaluate(() => new Promise<void>((resolve) => {
          requestAnimationFrame(() => requestAnimationFrame(() => resolve()))
        }))
        const neutral = await courtyardCenter(one, info, `live-round-${index}-neutral`)
        expect(opening.equals(neutral), `round ${currentRound} must show a real opening camera move`).toBe(false)
        if (index === 1) {
          await one.getByRole('region', { name: 'Grenadiers courtyard', exact: true }).click()
          await one.keyboard.press('Space')
          await expect.poll(() => game.frames.some((frame) => frame.type === 'snapshot'
            && frame.room.roundId === currentRound && frame.room.phase.type === 'result')).toBe(true)
        }
      }
    } catch (error) {
      const lifecycle = [game, peer].map((observed) => observed.frames
        .filter((frame) => 'room' in frame).slice(-30).map((frame) => {
          if (!('room' in frame)) return null
          return { type: frame.type, phase: frame.room.phase.type, roundId: frame.room.roundId,
            roundNumber: frame.room.roundNumber, tick: frame.room.tick,
            seats: frame.room.seats.map(({ seat, connection }) => ({ seat, connection })) }
        }))
      await info.attach('public-room-lifecycle', { body: JSON.stringify({ lifecycle, errors }), contentType: 'application/json' })
      throw error
    }
    expect(errors).toEqual([])
    expect(policy.violations).toEqual([])
    await one.getByRole('button', { name: 'Leave room', exact: true }).click()
    await two.getByRole('button', { name: 'Leave room', exact: true }).click()
  } finally { await Promise.all(contexts.map((owned) => owned.close())) }
})
