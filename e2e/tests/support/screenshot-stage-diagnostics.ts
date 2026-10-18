import { createRequire } from 'node:module'
import { performance } from 'node:perf_hooks'
import type { Page } from '@playwright/test'

const PLAYWRIGHT_VERSION = '1.63.0'
const UNSUPPORTED_MESSAGE = 'Screenshot stage diagnostics require Playwright 1.63.0 private APIs'

export const SCREENSHOT_STAGE_LIMIT = 128

type ScreenshotStage = 'takingPageScreenshot' | 'originalViewportSize'
  | 'preparePageForScreenshot' | 'waitingForFonts' | 'fontsLoaded' | 'fullPageSize'
  | 'getLayoutMetrics' | 'captureScreenshot' | 'restorePageAfterScreenshot'

export type ScreenshotStageEvent = {
  stage: ScreenshotStage
  boundary: 'started' | 'finished'
  pageOrdinal: number
  captureOrdinal: number
  nodeAt: number
  success: boolean
  error: boolean
}

type AnyFunction = (this: unknown, ...args: unknown[]) => unknown
type Instrumentation = {
  addListener: (listener: object, context: object) => void
  removeListener: (listener: object) => void
}
type Screenshotter = Record<string, unknown> & { _page: unknown }
type ProtocolClient = Record<string, unknown> & { send: AnyFunction }
type ServerPage = {
  screenshotter: Screenshotter
  instrumentation: Instrumentation
  browserContext: object
  delegate: { _mainFrameSession: { _client: ProtocolClient } }
}
type ClientPage = Page & {
  _connection?: { toImpl?: (value: unknown) => unknown }
}
type CaptureContext = {
  values: ScreenshotStageEvent[]
  pageOrdinal: number
  captureOrdinal: number
  active: boolean
}

const require = createRequire(import.meta.url)

function retain(values: ScreenshotStageEvent[], value: ScreenshotStageEvent): void {
  values.push(value)
  if (values.length > SCREENSHOT_STAGE_LIMIT) values.shift()
}

function record(context: CaptureContext, stage: ScreenshotStage,
  boundary: ScreenshotStageEvent['boundary'], success: boolean, error: boolean): void {
  if (!context.active) return
  retain(context.values, {
    stage, boundary, pageOrdinal: context.pageOrdinal, captureOrdinal: context.captureOrdinal,
    nodeAt: performance.now(), success, error,
  })
}

function isPromise(value: unknown): value is Promise<unknown> {
  return value instanceof Promise
}

function observeResult(context: CaptureContext, stage: ScreenshotStage, result: unknown): void {
  if (!isPromise(result)) {
    record(context, stage, 'finished', true, false)
    return
  }
  void result.then(
    () => { record(context, stage, 'finished', true, false) },
    () => { record(context, stage, 'finished', false, true) },
  )
}

function replaceMethod(target: Record<string, unknown>, name: string, stage: ScreenshotStage,
  context: CaptureContext,
  shouldRecord: (args: unknown[]) => boolean = () => true): () => void {
  const ownDescriptor = Object.getOwnPropertyDescriptor(target, name)
  if (ownDescriptor && !ownDescriptor.configurable) throw new Error(UNSUPPORTED_MESSAGE)
  if (!ownDescriptor && !Object.isExtensible(target)) throw new Error(UNSUPPORTED_MESSAGE)
  const original = target[name]
  if (typeof original !== 'function') throw new Error(UNSUPPORTED_MESSAGE)
  const wrapped = function (this: unknown, ...args: unknown[]): unknown {
    if (!shouldRecord(args)) return Reflect.apply(original, this, args)
    record(context, stage, 'started', false, false)
    let result: unknown
    try {
      result = Reflect.apply(original, this, args)
    } catch (error) {
      record(context, stage, 'finished', false, true)
      throw error
    }
    observeResult(context, stage, result)
    return result
  }
  Object.defineProperty(target, name, {
    configurable: true, enumerable: ownDescriptor?.enumerable ?? false,
    writable: true, value: wrapped,
  })
  return () => {
    if (ownDescriptor) Object.defineProperty(target, name, ownDescriptor)
    else delete target[name]
  }
}

function validateServerPage(value: unknown): ServerPage {
  if (typeof value !== 'object' || value === null) throw new Error(UNSUPPORTED_MESSAGE)
  const candidate = value as Partial<ServerPage>
  const screenshotter = candidate.screenshotter
  const instrumentation = candidate.instrumentation
  const context = candidate.browserContext
  const client = candidate.delegate?._mainFrameSession?._client
  const hookNames = [
    '_originalViewportSize', '_preparePageForScreenshot', '_fullPageSize',
    '_restorePageAfterScreenshot',
  ]
  if (!screenshotter || screenshotter._page !== value
    || !hookNames.every((name) => typeof screenshotter[name] === 'function')
    || !instrumentation || typeof instrumentation.addListener !== 'function'
    || typeof instrumentation.removeListener !== 'function'
    || typeof context !== 'object' || context === null
    || !client || typeof client.send !== 'function') throw new Error(UNSUPPORTED_MESSAGE)
  return candidate as ServerPage
}

export function resolveScreenshotServerPage(page: Page): ServerPage {
  const packageValue = require('playwright-core/package.json') as { version?: unknown }
  if (packageValue.version !== PLAYWRIGHT_VERSION) throw new Error(UNSUPPORTED_MESSAGE)
  const clientPage = page as ClientPage
  if (typeof clientPage._connection?.toImpl !== 'function') throw new Error(UNSUPPORTED_MESSAGE)
  return validateServerPage(clientPage._connection.toImpl(page))
}

export function installScreenshotStageDiagnostics(serverPageValue: unknown,
  values: ScreenshotStageEvent[], pageOrdinal: number, captureOrdinal: number): () => void {
  const serverPage = validateServerPage(serverPageValue)
  const context = { values, pageOrdinal, captureOrdinal, active: true }
  const restorers: Array<() => void> = []
  let listenerInstalled = false
  let cleaned = false
  const progressStages = new Map<string, ScreenshotStage>([
    ['taking page screenshot', 'takingPageScreenshot'],
    ['waiting for fonts to load...', 'waitingForFonts'],
    ['fonts loaded', 'fontsLoaded'],
  ])
  const listener = {
    onCallLog(sdkObject: unknown, _metadata: unknown, _logName: unknown, message: unknown) {
      const attributedPage = typeof sdkObject === 'object' && sdkObject !== null
        ? (sdkObject as { attribution?: { page?: unknown } }).attribution?.page : undefined
      const stage = typeof message === 'string' ? progressStages.get(message) : undefined
      if (attributedPage !== serverPage || !stage) return
      record(context, stage, 'started', false, false)
      record(context, stage, 'finished', true, false)
    },
  }
  const cleanup = (): void => {
    context.active = false
    if (cleaned) return
    cleaned = true
    if (listenerInstalled) {
      try { serverPage.instrumentation.removeListener(listener) } catch { /* Best-effort diagnostic cleanup. */ }
    }
    for (const restore of restorers.reverse()) {
      try { restore() } catch { /* Best-effort diagnostic cleanup. */ }
    }
  }
  try {
    const screenshotter = serverPage.screenshotter
    restorers.push(replaceMethod(screenshotter, '_originalViewportSize', 'originalViewportSize',
      context))
    restorers.push(replaceMethod(screenshotter, '_preparePageForScreenshot',
      'preparePageForScreenshot', context))
    restorers.push(replaceMethod(screenshotter, '_fullPageSize', 'fullPageSize',
      context))
    restorers.push(replaceMethod(screenshotter, '_restorePageAfterScreenshot',
      'restorePageAfterScreenshot', context))
    const client = serverPage.delegate._mainFrameSession._client
    restorers.push(replaceMethod(client, 'send', 'getLayoutMetrics', context,
      (args) => args[0] === 'Page.getLayoutMetrics'))
    restorers.push(replaceMethod(client, 'send', 'captureScreenshot', context,
      (args) => args[0] === 'Page.captureScreenshot'))
    listenerInstalled = true
    serverPage.instrumentation.addListener(listener, serverPage.browserContext)
    return cleanup
  } catch {
    cleanup()
    throw new Error(UNSUPPORTED_MESSAGE)
  }
}
