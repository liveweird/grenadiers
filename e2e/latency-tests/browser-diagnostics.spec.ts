import AxeBuilder from '@axe-core/playwright'
import { expect, test, type Browser, type Frame, type Page, type TestInfo } from '@playwright/test'
import { startBrowserStallDiagnostics } from '../tests/support/browser-stall-diagnostics'
import { readViewportFits } from '../tests/support/diagnostic-phases'
import { GameControlRecorder } from '../tests/support/duel'
import { runGuardedLegacyAxeAudit, type LegacyAxeAudit } from '../tests/support/guarded-axe-audit'
import {
  exerciseActualScreenshotStages, exerciseSyntheticScreenshotStages,
} from './screenshot-stage-fixtures'
import { exerciseCaptureProfileFixtures } from './capture-profile-fixtures'

function violationRuleIds(result: Awaited<ReturnType<AxeBuilder['analyze']>>): string[] {
  return result.violations.map((violation) => violation.id).sort()
}

async function exerciseGuardedLegacyAudit(browser: Browser): Promise<void> {
  const context = await browser.newContext()
  try {
    const page = await context.newPage()
    await page.setContent('<main><button id="unnamed"></button></main>')
    const current = await new AxeBuilder({ page }).analyze()
    const legacy = await runGuardedLegacyAxeAudit(page)
    expect(violationRuleIds(legacy)).toContain('button-name')
    expect(violationRuleIds(legacy)).toEqual(violationRuleIds(current))

    await page.setContent('<main><iframe title="existing"></iframe></main>')
    await expect.poll(() => page.frames().length).toBe(2)
    await expect(runGuardedLegacyAxeAudit(page)).rejects.toThrow(
      'Reconnect accessibility audit requires one frame before analysis',
    )

    await page.setContent('<main></main>')
    const duringAttachment: LegacyAxeAudit = async (auditPage) => {
      await auditPage.evaluate(() => {
        const frame = document.createElement('iframe')
        frame.title = 'transient'
        document.body.append(frame)
        frame.remove()
      })
      return legacy
    }
    await expect(runGuardedLegacyAxeAudit(page, duringAttachment)).rejects.toThrow(
      'Reconnect accessibility audit observed a frame attachment during analysis',
    )

    const listeners = new Set<(frame: Frame) => void>()
    const cleanupFailure = new Error('fixed-listener-cleanup-failure')
    const guardedPage = {
      frames: () => [{} as Frame],
      on: (_event: 'frameattached', listener: (frame: Frame) => void) => { listeners.add(listener) },
      off: (_event: 'frameattached', listener: (frame: Frame) => void) => {
        listeners.delete(listener)
        throw cleanupFailure
      },
    } as unknown as Page
    const primaryFailure = new Error('fixed-primary-audit-failure')
    await expect(runGuardedLegacyAxeAudit(
      guardedPage,
      async () => { throw primaryFailure },
    )).rejects.toBe(primaryFailure)
    expect(listeners.size).toBe(0)
    await expect(runGuardedLegacyAxeAudit(guardedPage, async () => legacy))
      .rejects.toBe(cleanupFailure)
    expect(listeners.size).toBe(0)
  } finally {
    await context.close()
  }
}

async function installFakeGl(page: Page): Promise<void> {
  await page.evaluate(() => {
    const slow = <T>(value: T): T => {
      const started = performance.now()
      while (performance.now() - started < 55) { /* Controlled slow native stand-in. */ }
      return value
    }
    const extension = {
      loseContext(value: unknown) {
        if (this !== extension) throw new Error('receiver changed')
        return slow(value)
      },
    }
    class FakeGlContext {
      useProgram(value: unknown) { return slow(value) }
      deleteBuffer(value: unknown): never { throw value }
      deleteQuery(value: unknown) { return slow(value) }
      deleteSampler(value: unknown) { return slow(value) }
      deleteSync(value: unknown) { return slow(value) }
      deleteTransformFeedback(value: unknown) { return slow(value) }
      drawRangeElements(value: unknown) { return slow(value) }
      clear(): number { return 17 }
      getExtension(name: string): object | null {
        return name === 'WEBGL_lose_context' ? extension : null
      }
    }
    const target = window as typeof window & {
      WebGLRenderingContext: typeof FakeGlContext
      FakeGlContext: typeof FakeGlContext
    }
    Object.defineProperty(target, 'WebGLRenderingContext', {
      configurable: true, value: FakeGlContext,
    })
    target.FakeGlContext = FakeGlContext
  })
}

async function exerciseFakeGl(page: Page): Promise<object> {
  return page.evaluate(() => {
    const target = window as typeof window & {
      FakeGlContext: new () => {
        useProgram: (value: object) => object
        deleteBuffer: (value: object) => never
        deleteQuery: (value: object) => object
        deleteSampler: (value: object) => object
        deleteSync: (value: object) => object
        deleteTransformFeedback: (value: object) => object
        drawRangeElements: (value: object) => object
        clear: () => number
        getExtension: (name: string) => { loseContext: (value: object) => object }
      }
    }
    const gl = new target.FakeGlContext()
    const value = {}
    const failure = {}
    let sameException = false
    try { gl.deleteBuffer(failure) } catch (error) { sameException = error === failure }
    const first = gl.getExtension('WEBGL_lose_context')
    const second = gl.getExtension('WEBGL_lose_context')
    return {
      sameResult: gl.useProgram(value) === value,
      expandedResults: [
        gl.deleteQuery(value), gl.deleteSampler(value), gl.deleteSync(value),
        gl.deleteTransformFeedback(value), gl.drawRangeElements(value),
      ].every((result) => result === value),
      sameException,
      clearResult: gl.clear(),
      sameExtension: first === second,
      sameExtensionResult: first.loseContext(value) === value,
    }
  })
}

async function publishSyntheticGl(page: Page, credential: string, socketUrl: string): Promise<void> {
  await page.evaluate(async ({ rawCredential, rawSocketUrl }) => {
    const binding = (window as typeof window & {
      grenadiersStallDiagnostic: (event: object) => Promise<void>
    }).grenadiersStallDiagnostic
    for (let index = 1; index <= 70; index += 1) {
      await binding({
        label: 'glCall', method: 'drawArrays', contextOrdinal: 9, callCount: index,
        pageAt: performance.now(), documentOrdinal: 2, timeOrigin: performance.timeOrigin,
        duration: 50,
        stack: [{
          functionName: 'bad name',
          asset: 'https://private.invalid/bundle.js?token=private#fragment',
          line: 3, column: 4, raw: rawCredential,
        }],
        rawUrl: rawSocketUrl,
      })
    }
  }, { rawCredential: credential, rawSocketUrl: socketUrl })
}

async function publishSyntheticHeartbeats(page: Page, credential: string, socketUrl: string): Promise<void> {
  await page.evaluate(async ({ rawCredential, rawSocketUrl }) => {
    const binding = (window as typeof window & {
      grenadiersStallDiagnostic: (event: object) => Promise<void>
    }).grenadiersStallDiagnostic
    for (let index = 0; index < 300; index += 1) {
      await binding({
        label: 'heartbeat', pageAt: performance.now(), documentOrdinal: 2,
        timeOrigin: performance.timeOrigin, rawCredential, rawSocketUrl,
      })
    }
  }, { rawCredential: credential, rawSocketUrl: socketUrl })
}

test('game control diagnostics stay bounded and exclude transport secrets', async ({ browser }) => {
  await exerciseGuardedLegacyAudit(browser)
  await exerciseSyntheticScreenshotStages()
  const recorder = new GameControlRecorder()
  for (let index = 0; index < 100; index += 1) recorder.addSocket()
  const socket = recorder.addSocket()
  const credential = 'session-secret-that-must-not-appear'
  const capability = 'resume-capability-that-must-not-appear'
  const socketUrl = `wss://example.invalid/game/ws?token=${credential}`

  recorder.recordSent(socket, JSON.stringify({
    type: 'authenticate', protocolVersion: 16, token: credential, socketUrl,
  }))
  for (let sequence = 1; sequence <= 300; sequence += 1) {
    recorder.recordSent(socket, JSON.stringify({
      type: 'ready', sequence, roomId: capability, ready: true,
    }))
  }
  recorder.recordReceived(socket, JSON.stringify({
    type: 'admitted', sequence: 301, resumeToken: capability, capability,
  }))
  for (let revision = 1; revision <= 600; revision += 1) {
    recorder.recordReceived(socket, JSON.stringify({
      type: 'snapshot', room: { tick: revision + 100, revision }, resumeToken: capability,
    }))
  }
  recorder.recordReceived(socket, JSON.stringify({
    type: 'error', code: 'resumeRejected', sequence: 302, message: credential,
  }))
  recorder.recordReceived(socket, JSON.stringify({
    type: 'seatReplaced', roomId: capability,
  }))
  recorder.recordClose(socket)

  expect(recorder.diagnostics.events).toHaveLength(256)
  expect(recorder.diagnostics.socketStates).toHaveLength(64)
  expect(recorder.diagnostics.socketStates.at(-1)?.socket).toBe(socket)
  expect(recorder.diagnostics.events.slice(-4)).toMatchObject([
    { type: 'admitted', sequence: 301, admitted: true },
    { type: 'error', sequence: 302, errorCode: 'resumeRejected' },
    { type: 'seatReplaced', seatReplaced: true },
    { type: 'close', direction: 'transport' },
  ])
  expect(recorder.diagnostics.socketStates.at(-1)?.lastSnapshot).toMatchObject({
    tick: 700, revision: 600,
  })
  const publicReport = JSON.stringify(recorder.diagnostics)
  for (const secret of [credential, capability, socketUrl, 'message'])
    expect(publicReport).not.toContain(secret)

  const diagnostics = startBrowserStallDiagnostics()
  const context = await browser.newContext()
  try {
    const page = await context.newPage()
    await exerciseCaptureProfileFixtures(page).then(() => installFakeGl(page))
    await diagnostics.observePage(page, 'visibility-cover')
    await expect.poll(() => diagnostics.summary().pages[0].stateEvents
      .filter((event) => event.label === 'initialized').length).toBe(1)
    await page.setContent('<title>Diagnostic lifecycle</title>')
    await expect.poll(() => diagnostics.summary().pages[0].stateEvents
      .filter((event) => event.label === 'initialized').length).toBe(2)
    const initialized = diagnostics.summary().pages[0].stateEvents
      .filter((event) => event.label === 'initialized')
    expect(initialized.map((event) => event.documentOrdinal)).toEqual([1, 2])
    expect(initialized.every((event) => event.timeOrigin > 0)).toBe(true)
    await expect.poll(() => diagnostics.summary().pages[0].activityEvents
      .some((event) => event.label === 'heartbeat')).toBe(true)

    expect(await diagnostics.phase('width', 1, async () => readViewportFits(page))).toBe(true)
    await expect.poll(() => diagnostics.summary().pages[0].activityEvents
      .filter((event) => ['page-enter', 'read-start', 'read-end'].includes(event.label))
      .map((event) => event.label)).toEqual(['page-enter', 'read-start', 'read-end'])
    const widthPagePhases = diagnostics.summary().pages[0].activityEvents
      .filter((event) => ['page-enter', 'read-start', 'read-end'].includes(event.label))
    expect(widthPagePhases.every((event) => event.pageAt >= 0
      && event.timeOrigin > 0 && event.nodeAt >= 0)).toBe(true)
    expect(diagnostics.summary().phases.slice(-2)).toMatchObject([
      { label: 'width', boundary: 'sent', pageOrdinal: 1 },
      { label: 'width', boundary: 'returned', pageOrdinal: 1 },
    ])
    expect(diagnostics.summary().phases.at(-2)!.nodeAt).toBeLessThanOrEqual(diagnostics.summary().phases.at(-1)!.nodeAt)

    await exerciseActualScreenshotStages(page, diagnostics)

    const temporaryPage = await context.newPage()
    await temporaryPage.close()
    expect(diagnostics.summary().temporaryPages).toMatchObject([
      { ordinal: 1, boundary: 'opened' }, { ordinal: 1, boundary: 'closed' },
    ])
    expect(diagnostics.summary().temporaryPages[0].nodeAt)
      .toBeLessThanOrEqual(diagnostics.summary().temporaryPages[1].nodeAt)
    const transparency = await exerciseFakeGl(page)
    expect(transparency).toEqual({
      sameResult: true, expandedResults: true, sameException: true, clearResult: 17,
      sameExtension: true, sameExtensionResult: true,
    })
    await expect.poll(() => diagnostics.summary().pages[0].glCalls.map((call) => call.method))
      .toEqual(expect.arrayContaining([
        'useProgram', 'deleteQuery', 'deleteSampler', 'deleteSync', 'deleteTransformFeedback',
        'drawRangeElements', 'loseContext',
      ]))
    expect(diagnostics.summary().pages[0].glCalls
      .filter((call) => call.method === 'useProgram' || call.method === 'loseContext')
      .map((call) => call.callCount)).toEqual([1, 1])

    await publishSyntheticGl(page, credential, socketUrl)
    const boundedGl = diagnostics.summary().pages[0].glCalls
    expect(boundedGl).toHaveLength(64)
    expect(boundedGl.at(-1)).toMatchObject({
      method: 'drawArrays', contextOrdinal: 9, callCount: 70, duration: 50,
      stack: [{ functionName: 'anonymous', asset: 'bundle.js', line: 3, column: 4 }],
    })
    expect(JSON.stringify(boundedGl)).not.toContain('private')
    expect(JSON.stringify(boundedGl)).not.toContain('token')
    expect(JSON.stringify(boundedGl)).not.toContain('http')

    await publishSyntheticHeartbeats(page, credential, socketUrl)
    const boundedActivity = diagnostics.summary().pages[0].activityEvents
    expect(boundedActivity).toHaveLength(256)
    expect(boundedActivity.filter((event) => event.label === 'heartbeat').length).toBeGreaterThanOrEqual(250)

    for (let index = 0; index < 70; index += 1) {
      await diagnostics.phase(index % 2 === 0 ? 'axe' : 'screenshot', 1, async () => undefined)
    }
    expect(diagnostics.summary().phases).toHaveLength(128)
    expect(diagnostics.summary().phases.every((phase) => ['axe', 'width', 'screenshot']
      .includes(phase.label) && ['sent', 'returned'].includes(phase.boundary))).toBe(true)
    const phaseFailure = new Error('fixed-phase-failure')
    let retainedPhaseFailure: unknown
    try {
      await diagnostics.phase('axe', 1, async () => { throw phaseFailure })
    } catch (error) { retainedPhaseFailure = error }
    expect(retainedPhaseFailure).toBe(phaseFailure)
    expect(diagnostics.summary().phases.slice(-2)).toMatchObject([
      { label: 'axe', boundary: 'sent' }, { label: 'axe', boundary: 'returned' },
    ])
    expect(JSON.stringify(diagnostics.summary())).not.toContain(credential)
    expect(JSON.stringify(diagnostics.summary())).not.toContain(capability)
    expect(JSON.stringify(diagnostics.summary())).not.toContain(socketUrl)

    const failingPage = {
      exposeBinding: async () => undefined,
      addInitScript: async () => { throw new Error(credential) },
      evaluate: async () => { throw new Error(capability) },
      context: () => ({ newCDPSession: async () => { throw new Error(socketUrl) } }),
      on: () => undefined,
      off: () => undefined,
    } as unknown as Page
    await expect(diagnostics.observePage(failingPage, 'reconnect-primary')).resolves.toBeUndefined()
    expect(diagnostics.summary().pages[1].failures.map((failure) => failure.stage)).toEqual([
      'futureDocumentSetup', 'currentDocumentSetup', 'windowAssociation',
    ])
    expect(JSON.stringify(diagnostics.summary())).not.toContain(credential)

    const annotations: TestInfo['annotations'] = []
    const failingTestInfo = {
      annotations,
      attach: async () => { throw new Error(capability) },
    } as unknown as TestInfo
    await expect(diagnostics.attach(failingTestInfo, 'diagnostic-test')).resolves.toBeUndefined()
    expect(annotations).toEqual([{
      type: 'browser-stall-diagnostics', description: 'attachment-failed',
    }])
    diagnostics.stop()
    await expect(diagnostics.observePage(failingPage, 'reconnect-peer')).resolves.toBeUndefined()
    expect(diagnostics.summary().failures.at(-1)?.stage).toBe('observeAfterStop')
  } finally {
    await context.close()
    diagnostics.stop()
  }
})
