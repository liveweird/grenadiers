import { expect, test, type Page, type TestInfo } from '@playwright/test'
import { startBrowserStallDiagnostics } from '../tests/support/browser-stall-diagnostics'
import { GameControlRecorder } from '../tests/support/duel'

test('game control diagnostics stay bounded and exclude transport secrets', async ({ browser }) => {
  const recorder = new GameControlRecorder()
  for (let index = 0; index < 100; index += 1) recorder.addSocket()
  const socket = recorder.addSocket()
  const credential = 'session-secret-that-must-not-appear'
  const capability = 'resume-capability-that-must-not-appear'
  const socketUrl = `wss://example.invalid/game/ws?token=${credential}`

  recorder.recordSent(socket, JSON.stringify({
    type: 'authenticate', protocolVersion: 7, token: credential, socketUrl,
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
