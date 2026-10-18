import { expect, type Page } from '@playwright/test'
import { BrowserStallDiagnostics } from '../tests/support/browser-stall-diagnostics'
import {
  installScreenshotStageDiagnostics, SCREENSHOT_STAGE_LIMIT,
  type ScreenshotStageEvent,
} from '../tests/support/screenshot-stage-diagnostics'

type HookName = '_originalViewportSize' | '_preparePageForScreenshot' | '_fullPageSize'
  | '_restorePageAfterScreenshot'

type FakeInstrumentation = {
  listeners: Set<{ onCallLog: (...args: unknown[]) => void }>
  addListener: (listener: { onCallLog: (...args: unknown[]) => void }, context: object) => void
  removeListener: (listener: { onCallLog: (...args: unknown[]) => void }) => void
}

function fakeServerPage() {
  const receiverChecks: boolean[] = []
  const receivedArguments: unknown[][] = []
  const inheritedPromise = Promise.resolve('fixed-viewport')
  const rejectedError = new Error('fixed-hook-rejection')
  const rejectedPromise = Promise.reject(rejectedError)
  void rejectedPromise.catch(() => undefined)
  const syncError = new Error('fixed-hook-sync-error')
  const prototype = {
    _originalViewportSize(this: unknown, ...args: unknown[]) {
      receiverChecks.push(this === screenshotter)
      receivedArguments.push(args)
      return inheritedPromise
    },
  }
  const screenshotter = Object.create(prototype) as Record<HookName | '_page', unknown>
  Object.defineProperty(screenshotter, '_preparePageForScreenshot', {
    configurable: true, enumerable: true, writable: false,
    value(this: unknown, ...args: unknown[]) {
      receiverChecks.push(this === screenshotter)
      receivedArguments.push(args)
      return 'fixed-prepare-result'
    },
  })
  screenshotter._fullPageSize = function (this: unknown, ...args: unknown[]) {
    receiverChecks.push(this === screenshotter)
    receivedArguments.push(args)
    return rejectedPromise
  }
  screenshotter._restorePageAfterScreenshot = function (this: unknown, ...args: unknown[]) {
    receiverChecks.push(this === screenshotter)
    receivedArguments.push(args)
    throw syncError
  }
  const clientCalls: unknown[][] = []
  const capturePromise = Promise.resolve({ data: 'fixed-secret-result' })
  const client = {
    send(this: unknown, ...args: unknown[]) {
      receiverChecks.push(this === client)
      clientCalls.push(args)
      return capturePromise
    },
  }
  const listeners = new Set<{ onCallLog: (...args: unknown[]) => void }>()
  const browserContext = {}
  const instrumentation: FakeInstrumentation = {
    listeners,
    addListener(listener, context) {
      expect(context).toBe(browserContext)
      listeners.add(listener)
    },
    removeListener(listener) { listeners.delete(listener) },
  }
  const serverPage = {
    screenshotter, instrumentation, browserContext,
    delegate: { _mainFrameSession: { _client: client } },
    attribution: { page: undefined as unknown },
  }
  screenshotter._page = serverPage
  serverPage.attribution.page = serverPage
  return {
    serverPage, screenshotter, client, instrumentation, receiverChecks, receivedArguments,
    clientCalls, inheritedPromise, rejectedPromise, rejectedError, syncError, capturePromise,
  }
}

async function exerciseScreenshotStagePassThrough(): Promise<void> {
  const fake = fakeServerPage()
  const values: ScreenshotStageEvent[] = []
  const inheritedBefore = Object.getOwnPropertyDescriptor(fake.screenshotter, '_originalViewportSize')
  const prepareBefore = Object.getOwnPropertyDescriptor(fake.screenshotter, '_preparePageForScreenshot')
  const sendBefore = Object.getOwnPropertyDescriptor(fake.client, 'send')
  const cleanup = installScreenshotStageDiagnostics(fake.serverPage, values, 7, 11)
  expect(fake.instrumentation.listeners.size).toBe(1)

  const viewportArgument = { fixed: 'viewport-argument' }
  const viewportResult = (fake.screenshotter._originalViewportSize as (
    value: object) => Promise<string>)(viewportArgument)
  expect(viewportResult).toBe(fake.inheritedPromise)
  expect((fake.screenshotter._preparePageForScreenshot as (value: number) => string)(19))
    .toBe('fixed-prepare-result')
  const rejectedResult = (fake.screenshotter._fullPageSize as () => Promise<never>)()
  expect(rejectedResult).toBe(fake.rejectedPromise)
  await expect(rejectedResult).rejects.toBe(fake.rejectedError)
  expect(() => (fake.screenshotter._restorePageAfterScreenshot as () => void)())
    .toThrow(fake.syncError)

  const secretParameters = { credential: 'fixed-secret-parameter' }
  const captureResult = fake.client.send('Page.captureScreenshot', secretParameters)
  expect(captureResult).toBe(fake.capturePromise)
  expect(fake.client.send('Runtime.evaluate', secretParameters)).toBe(fake.capturePromise)
  for (const listener of fake.instrumentation.listeners) {
    listener.onCallLog(fake.serverPage, { secret: 'fixed-secret-metadata' }, 'api',
      'taking page screenshot')
    listener.onCallLog(fake.serverPage, {}, 'api', 'waiting for fonts to load...')
    listener.onCallLog(fake.serverPage, {}, 'api', 'fonts loaded')
    listener.onCallLog({ attribution: { page: {} } }, {}, 'api', 'taking page screenshot')
    listener.onCallLog(fake.serverPage, {}, 'api', 'unrelated progress')
  }
  await Promise.resolve()

  expect(fake.receiverChecks.every(Boolean)).toBe(true)
  expect(fake.receivedArguments).toEqual([[viewportArgument], [19], [], []])
  expect(fake.clientCalls).toEqual([
    ['Page.captureScreenshot', secretParameters], ['Runtime.evaluate', secretParameters],
  ])
  expect(values.map((event) => event.stage)).toEqual(expect.arrayContaining([
    'originalViewportSize', 'preparePageForScreenshot', 'fullPageSize',
    'restorePageAfterScreenshot', 'captureScreenshot', 'takingPageScreenshot',
    'waitingForFonts', 'fontsLoaded',
  ]))
  expect(values.filter((event) => event.boundary === 'finished' && event.error)
    .map((event) => event.stage)).toEqual(['fullPageSize', 'restorePageAfterScreenshot'])
  expect(values.every((event) => event.pageOrdinal === 7 && event.captureOrdinal === 11
    && event.nodeAt >= 0)).toBe(true)
  expect(JSON.stringify(values)).not.toContain('fixed-secret')

  cleanup()
  cleanup()
  expect(fake.instrumentation.listeners.size).toBe(0)
  expect(Object.getOwnPropertyDescriptor(fake.screenshotter, '_originalViewportSize'))
    .toEqual(inheritedBefore)
  expect(Object.getOwnPropertyDescriptor(fake.screenshotter, '_preparePageForScreenshot'))
    .toEqual(prepareBefore)
  expect(Object.getOwnPropertyDescriptor(fake.client, 'send')).toEqual(sendBefore)
}

async function exerciseScreenshotStageBoundsAndUnsupported(): Promise<void> {
  const fake = fakeServerPage()
  const values: ScreenshotStageEvent[] = []
  const cleanup = installScreenshotStageDiagnostics(fake.serverPage, values, 2, 3)
  for (let index = 0; index < SCREENSHOT_STAGE_LIMIT; index += 1) {
    fake.client.send('Page.getLayoutMetrics', { index, secret: 'fixed-secret-bounded' })
  }
  await Promise.resolve()
  expect(values).toHaveLength(SCREENSHOT_STAGE_LIMIT)
  expect(values.every((event) => event.stage === 'getLayoutMetrics')).toBe(true)
  expect(JSON.stringify(values)).not.toContain('fixed-secret')
  cleanup()

  const unsupported = fakeServerPage()
  const original = unsupported.screenshotter._fullPageSize
  Reflect.deleteProperty(unsupported.screenshotter, '_fullPageSize')
  await expect(async () => installScreenshotStageDiagnostics(
    unsupported.serverPage, [], 1, 1,
  )).rejects.toThrow('Screenshot stage diagnostics require Playwright 1.63.0 private APIs')
  expect(unsupported.instrumentation.listeners.size).toBe(0)
  expect(unsupported.screenshotter._originalViewportSize)
    .toBe(Object.getPrototypeOf(unsupported.screenshotter)._originalViewportSize)
  unsupported.screenshotter._fullPageSize = original
}

async function exerciseScreenshotStageCleanupQuiescence(): Promise<void> {
  for (const outcome of ['resolve', 'reject'] as const) {
    const fake = fakeServerPage()
    const failure = new Error('fixed-late-rejection')
    let settle: () => void = () => undefined
    const pending = new Promise<string>((resolve, reject) => {
      settle = outcome === 'resolve' ? () => { resolve('fixed-late-result') }
        : () => { reject(failure) }
    })
    Object.defineProperty(fake.client, 'send', {
      configurable: true, enumerable: true, writable: true, value: () => pending,
    })
    const values: ScreenshotStageEvent[] = []
    const cleanup = installScreenshotStageDiagnostics(fake.serverPage, values, 4, 5)
    const listener = [...fake.instrumentation.listeners][0]
    const result = fake.client.send('Page.captureScreenshot', { secret: 'fixed-late-secret' })
    expect(result).toBe(pending)
    fake.instrumentation.removeListener = () => { throw new Error('fixed-remove-failure') }
    cleanup()
    const retained = JSON.stringify(values)
    listener.onCallLog(fake.serverPage, {}, 'api', 'fonts loaded')
    settle()
    if (outcome === 'resolve') await expect(result).resolves.toBe('fixed-late-result')
    else await expect(result).rejects.toBe(failure)
    await Promise.resolve()
    expect(JSON.stringify(values)).toBe(retained)
    expect(values).toMatchObject([{
      stage: 'captureScreenshot', boundary: 'started', success: false, error: false,
    }])
  }
}

async function exerciseFirstPipelineThenSecondary(): Promise<void> {
  const fake = fakeServerPage()
  const viewportResults = [Promise.resolve('first-viewport'), Promise.resolve('second-viewport')]
  const prepareResults = [Promise.resolve('first-prepare'), Promise.resolve('second-prepare')]
  const restoreResults = [Promise.resolve('first-restore'), Promise.resolve('second-restore')]
  const viewportCalls: unknown[][] = []
  const prepareCalls: unknown[][] = []
  let restoreCallCount = 0
  Object.defineProperty(fake.screenshotter, '_originalViewportSize', {
    configurable: true, value(this: unknown, ...args: unknown[]) {
      expect(this).toBe(fake.screenshotter)
      viewportCalls.push(args)
      return viewportResults[viewportCalls.length - 1]
    },
  })
  Object.defineProperty(fake.screenshotter, '_preparePageForScreenshot', {
    configurable: true, value(this: unknown, ...args: unknown[]) {
      expect(this).toBe(fake.screenshotter)
      prepareCalls.push(args)
      return prepareResults[prepareCalls.length - 1]
    },
  })
  Object.defineProperty(fake.screenshotter, '_restorePageAfterScreenshot', {
    configurable: true, value: () => restoreResults[restoreCallCount++],
  })
  const values: ScreenshotStageEvent[] = []
  const cleanup = installScreenshotStageDiagnostics(fake.serverPage, values, 6, 9)
  const restore = fake.screenshotter._restorePageAfterScreenshot as () => Promise<string>
  const firstViewport = (fake.screenshotter._originalViewportSize as (
    ...args: unknown[]) => unknown)('first-viewport-argument')
  expect(firstViewport).toBe(viewportResults[0])
  await firstViewport
  const firstPrepare = (fake.screenshotter._preparePageForScreenshot as (
    ...args: unknown[]) => unknown)('first-prepare-argument')
  expect(firstPrepare).toBe(prepareResults[0])
  await firstPrepare
  const firstRestore = restore()
  expect(firstRestore).toBe(restoreResults[0])
  await firstRestore
  const retained = JSON.stringify(values)

  const secondViewport = (fake.screenshotter._originalViewportSize as (
    ...args: unknown[]) => unknown)('second-viewport-argument')
  expect(secondViewport).toBe(viewportResults[1])
  await secondViewport
  const secondPrepare = (fake.screenshotter._preparePageForScreenshot as (
    ...args: unknown[]) => unknown)('second-prepare-argument')
  expect(secondPrepare).toBe(prepareResults[1])
  await secondPrepare
  const secondRestore = restore()
  expect(secondRestore).toBe(restoreResults[1])
  await secondRestore
  expect(JSON.stringify(values)).toBe(retained)
  expect(viewportCalls).toEqual([['first-viewport-argument'], ['second-viewport-argument']])
  expect(prepareCalls).toEqual([['first-prepare-argument'], ['second-prepare-argument']])
  expect(restoreCallCount).toBe(2)
  cleanup()
}

async function exerciseFailedFirstPipelineThenSecondary(): Promise<void> {
  const fake = fakeServerPage()
  const firstFailure = new Error('fixed-first-prepare-failure')
  const firstPrepare = Promise.reject(firstFailure)
  const secondPrepare = Promise.resolve('fixed-secondary-prepare')
  void firstPrepare.catch(() => undefined)
  let prepareCallCount = 0
  let restoreCallCount = 0
  Object.defineProperty(fake.screenshotter, '_preparePageForScreenshot', {
    configurable: true, value: () => [firstPrepare, secondPrepare][prepareCallCount++],
  })
  Object.defineProperty(fake.screenshotter, '_restorePageAfterScreenshot', {
    configurable: true, value: () => {
      restoreCallCount += 1
      return Promise.resolve('fixed-restore')
    },
  })
  const values: ScreenshotStageEvent[] = []
  const cleanup = installScreenshotStageDiagnostics(fake.serverPage, values, 8, 10)
  const prepare = fake.screenshotter._preparePageForScreenshot as () => Promise<unknown>
  const restore = fake.screenshotter._restorePageAfterScreenshot as () => Promise<string>
  await (fake.screenshotter._originalViewportSize as () => Promise<string>)()
  for (const listener of fake.instrumentation.listeners)
    listener.onCallLog(fake.serverPage, {}, 'api', 'taking page screenshot')
  const firstResult = prepare()
  expect(firstResult).toBe(firstPrepare)
  await expect(firstResult).rejects.toBe(firstFailure)
  expect(values).toEqual(expect.arrayContaining([
    expect.objectContaining({ stage: 'preparePageForScreenshot', boundary: 'finished', error: true }),
  ]))
  await restore()
  const retained = JSON.stringify(values)

  for (const listener of fake.instrumentation.listeners)
    listener.onCallLog(fake.serverPage, {}, 'api', 'taking page screenshot')
  await (fake.screenshotter._originalViewportSize as () => Promise<string>)()
  const secondResult = prepare()
  expect(secondResult).toBe(secondPrepare)
  await secondResult
  await restore()
  expect(prepareCallCount).toBe(2)
  expect(restoreCallCount).toBe(2)
  expect(JSON.stringify(values)).toBe(retained)
  cleanup()
}

async function exercisePendingFirstPipelineThenSecondary(): Promise<void> {
  const fake = fakeServerPage()
  let resolveFirst!: (value: string) => void
  const firstViewport = new Promise<string>((resolve) => { resolveFirst = resolve })
  const secondViewport = Promise.resolve('fixed-secondary-viewport')
  const calls: unknown[][] = []
  Object.defineProperty(fake.screenshotter, '_originalViewportSize', {
    configurable: true, value(this: unknown, ...args: unknown[]) {
      expect(this).toBe(fake.screenshotter)
      calls.push(args)
      return calls.length === 1 ? firstViewport : secondViewport
    },
  })
  const values: ScreenshotStageEvent[] = []
  const cleanup = installScreenshotStageDiagnostics(fake.serverPage, values, 3, 12)
  const firstResult = (fake.screenshotter._originalViewportSize as (
    ...args: unknown[]) => Promise<string>)('first-pending')
  expect(firstResult).toBe(firstViewport)
  const secondResult = (fake.screenshotter._originalViewportSize as (
    ...args: unknown[]) => Promise<string>)('secondary-while-pending')
  expect(secondResult).toBe(secondViewport)
  await expect(secondResult).resolves.toBe('fixed-secondary-viewport')
  const retained = JSON.stringify(values)
  resolveFirst('fixed-first-viewport')
  await expect(firstResult).resolves.toBe('fixed-first-viewport')
  await Promise.resolve()
  expect(JSON.stringify(values)).toBe(retained)
  expect(values).toMatchObject([{
    stage: 'originalViewportSize', boundary: 'started', success: false, error: false,
  }])
  expect(calls).toEqual([['first-pending'], ['secondary-while-pending']])
  cleanup()
}

export async function exerciseSyntheticScreenshotStages(): Promise<void> {
  await exerciseScreenshotStagePassThrough()
  await exerciseScreenshotStageBoundsAndUnsupported()
  await exerciseScreenshotStageCleanupQuiescence()
  await exerciseFirstPipelineThenSecondary()
  await exerciseFailedFirstPipelineThenSecondary()
  await exercisePendingFirstPipelineThenSecondary()
}

export async function exerciseActualScreenshotStages(page: Page,
  diagnostics: BrowserStallDiagnostics): Promise<void> {
  await page.setContent('<main style="min-height: 900px">Safe screenshot stage fixture</main>')
  const screenshot = await diagnostics.screenshot(page, 1,
    async () => page.screenshot({ fullPage: true }))
  expect(screenshot.subarray(0, 8)).toEqual(Buffer.from([137, 80, 78, 71, 13, 10, 26, 10]))
  const events = diagnostics.summary().screenshotStages
  expect(new Set(events.map((event) => event.stage))).toEqual(new Set([
    'takingPageScreenshot', 'originalViewportSize', 'preparePageForScreenshot',
    'waitingForFonts', 'fontsLoaded', 'fullPageSize', 'getLayoutMetrics',
    'captureScreenshot', 'restorePageAfterScreenshot',
  ]))
  expect(events.every((event) => event.pageOrdinal === 1 && event.captureOrdinal === 1)).toBe(true)
  expect(events.filter((event) => event.boundary === 'finished')
    .every((event) => event.success && !event.error)).toBe(true)

  const failure = new Error('fixed-screenshot-action-failure')
  await expect(diagnostics.screenshot(page, 1, async () => { throw failure })).rejects.toBe(failure)
  expect(diagnostics.summary().phases.slice(-2)).toMatchObject([
    { label: 'screenshot', boundary: 'sent' },
    { label: 'screenshot', boundary: 'returned' },
  ])
}
