import { expect, test, type Page, type TestInfo } from '@playwright/test'
import { startBrowserStallDiagnostics } from '../tests/support/browser-stall-diagnostics'
import { GameControlRecorder } from '../tests/support/duel'

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

test('game control diagnostics stay bounded and exclude transport secrets', async ({ browser }) => {
  const recorder = new GameControlRecorder()
  for (let index = 0; index < 100; index += 1) recorder.addSocket()
  const socket = recorder.addSocket()
  const credential = 'session-secret-that-must-not-appear'
  const capability = 'resume-capability-that-must-not-appear'
  const socketUrl = `wss://example.invalid/game/ws?token=${credential}`

  recorder.recordSent(socket, JSON.stringify({
    type: 'authenticate', protocolVersion: 6, token: credential, socketUrl,
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
  expect(publicReport).not.toContain(credential)
  expect(publicReport).not.toContain(capability)
  expect(publicReport).not.toContain(socketUrl)
  expect(publicReport).not.toContain('message')

  const diagnostics = startBrowserStallDiagnostics()
  const context = await browser.newContext()
  try {
    const page = await context.newPage()
    await installFakeGl(page)
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
  } finally {
    await context.close()
    diagnostics.stop()
  }
})
