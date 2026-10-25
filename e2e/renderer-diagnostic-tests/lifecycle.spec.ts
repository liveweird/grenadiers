import { performance } from 'node:perf_hooks'
import { chromium, expect, test, type Browser, type BrowserContext, type Page } from '@playwright/test'
import { readViewportFits } from '../tests/support/diagnostic-phases'
import { runGuardedLegacyAxeAudit } from '../tests/support/guarded-axe-audit'
import { startBrowserStallDiagnostics } from '../tests/support/browser-stall-diagnostics'
import { observeBrowserPolicy, type BrowserPolicyObservation } from '../tests/support/browser-policy'
import {
  assertCapturePreservedState, assertFixedState, attachLifecycle, guardStaticOrigin, matrixCell,
  finishLifecycle, initializeLifecycle, readFixedState, warmLifecycle, type LifecycleEvidence,
  type LifecycleTimestamp, type RequestGuard,
} from './support'

type PagePair = { primary: Page; peer: Page }
type ContextPair = { primary: BrowserContext; peer: BrowserContext }
const BASE_URL = process.env.RENDERER_DIAGNOSTICS_BASE_URL ?? 'http://127.0.0.1:5188'

async function auditAndMeasure(
  pages: PagePair,
  diagnostics: ReturnType<typeof startBrowserStallDiagnostics>,
): Promise<void> {
  for (const [index, page] of [pages.primary, pages.peer].entries()) {
    const ordinal = index + 1
    const audit = await diagnostics.phase('axe', ordinal, () => runGuardedLegacyAxeAudit(page))
    expect(audit.violations).toEqual([])
    expect(await diagnostics.phase('width', ordinal, () => readViewportFits(page))).toBe(true)
  }
}

test('real Pixi renderer lifecycle matrix cell remains observable during full-page capture',
  async ({ browserName }, testInfo) => {
    expect(browserName).toBe('chromium')
    const cell = matrixCell(testInfo.project.name)
    const diagnostics = startBrowserStallDiagnostics()
    const timestamps: LifecycleTimestamp[] = [{ stage: 'started', nodeAt: performance.now() }]
    const states: LifecycleEvidence = { preCapture: {}, postCapture: {} }
    const guards: RequestGuard[] = []
    const policies: BrowserPolicyObservation[] = []
    const contexts: BrowserContext[] = []
    let launched: Browser | undefined
    let primaryFailed = false
    let primaryFailure: unknown
    let cleanupFailed = false
    let cleanupFailure: unknown
    const retainCleanupFailure = (error: unknown): void => {
      if (!cleanupFailed) { cleanupFailed = true; cleanupFailure = error }
    }
    try {
      launched = await chromium.launch(testInfo.project.use.launchOptions)
      const primaryContext = await launched.newContext({
        viewport: { width: 1440, height: 1050 }, reducedMotion: 'no-preference',
        serviceWorkers: 'block',
      })
      contexts.push(primaryContext)
      const peerContext = await launched.newContext({
        viewport: { width: 320, height: 844 }, reducedMotion: 'reduce',
        serviceWorkers: 'block',
      })
      contexts.push(peerContext)
      policies.push(...await Promise.all([
        observeBrowserPolicy(primaryContext), observeBrowserPolicy(peerContext),
      ]))
      const contextPair: ContextPair = {
        primary: primaryContext,
        peer: peerContext,
      }
      guards.push(
        await guardStaticOrigin(contextPair.primary, BASE_URL),
        await guardStaticOrigin(contextPair.peer, BASE_URL),
      )
      const pages: PagePair = {
        primary: await contextPair.primary.newPage(),
        peer: await contextPair.peer.newPage(),
      }
      guards[0].installPageChecks(pages.primary)
      guards[1].installPageChecks(pages.peer)
      await diagnostics.observePage(pages.primary, 'renderer-primary')
      await diagnostics.observePage(pages.peer, 'renderer-peer')
      await Promise.all([pages.primary, pages.peer].map((page) => page.goto(new URL('/', BASE_URL).href)))
      timestamps.push({ stage: 'navigated', nodeAt: performance.now() })
      await expect(pages.primary.locator('[data-build-target]'))
        .toHaveAttribute('data-build-target', 'grenadiers-renderer-diagnostic')
      await Promise.all([
        initializeLifecycle(pages.primary, cell.primary),
        initializeLifecycle(pages.peer, cell.peer),
      ])
      timestamps.push({ stage: 'initialize-completed', nodeAt: performance.now() })
      await Promise.all([
        warmLifecycle(pages.primary, cell.primary),
        warmLifecycle(pages.peer, cell.peer),
      ])
      timestamps.push({ stage: 'warmup-completed', nodeAt: performance.now() })
      await Promise.all([
        finishLifecycle(pages.primary, cell.primary),
        finishLifecycle(pages.peer, cell.peer),
      ])
      timestamps.push({ stage: 'finish-applied', nodeAt: performance.now() })
      const first = await Promise.all([readFixedState(pages.primary), readFixedState(pages.peer)])
      await pages.primary.waitForTimeout(100)
      const second = await Promise.all([readFixedState(pages.primary), readFixedState(pages.peer)])
      expect(second[0].updates.count).toBe(first[0].updates.count)
      expect(second[1].updates.count).toBe(first[1].updates.count)
      expect(second[0].raf.count).toBeGreaterThan(first[0].raf.count)
      expect(second[1].raf.count).toBeGreaterThan(first[1].raf.count)
      states.preCapture.primary = second[0]
      states.preCapture.peer = second[1]
      assertFixedState(second[0], cell.primary, 'primary')
      assertFixedState(second[1], cell.peer, 'peer')
      expect(guards.flatMap((guard) => guard.unexpected)).toEqual([])
      expect(policies.flatMap((policy) => policy.violations)).toEqual([])
      await auditAndMeasure(pages, diagnostics)
      await diagnostics.screenshot(pages.primary, 1, () => pages.primary.screenshot({
        path: testInfo.outputPath(`${cell.name.replace('/', '-')}-primary-full-page.png`),
        fullPage: true,
      }))
      const postCapture = await Promise.all([
        readFixedState(pages.primary), readFixedState(pages.peer),
      ])
      states.postCapture.primary = postCapture[0]
      states.postCapture.peer = postCapture[1]
      assertFixedState(postCapture[0], cell.primary, 'primary')
      assertFixedState(postCapture[1], cell.peer, 'peer')
      assertCapturePreservedState(second[0], postCapture[0])
      assertCapturePreservedState(second[1], postCapture[1])
      expect(guards.flatMap((guard) => guard.unexpected)).toEqual([])
      expect(policies.flatMap((policy) => policy.violations)).toEqual([])
      timestamps.push({ stage: 'capture-completed', nodeAt: performance.now() })
    } catch (error) {
      primaryFailed = true
      primaryFailure = error
    } finally {
      try {
        await attachLifecycle(
          testInfo, cell, timestamps, states, guards.flatMap((guard) => guard.unexpected),
        )
      } catch (error) { retainCleanupFailure(error) }
      try { diagnostics.stop() } catch (error) { retainCleanupFailure(error) }
      try {
        await diagnostics.attach(testInfo, 'renderer-browser-stall-diagnostics')
      } catch (error) { retainCleanupFailure(error) }
      const closed = await Promise.allSettled(contexts.map((context) => context.close()))
      for (const result of closed) {
        if (result.status === 'rejected') retainCleanupFailure(result.reason)
      }
      try { await launched?.close() } catch (error) { retainCleanupFailure(error) }
    }
    if (primaryFailed) {
      if (cleanupFailed) testInfo.annotations.push({
        type: 'renderer-cleanup', description: 'failed-after-primary-error',
      })
      throw primaryFailure
    }
    if (cleanupFailed) throw cleanupFailure
  })
