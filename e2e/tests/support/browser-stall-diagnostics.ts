import { performance } from 'node:perf_hooks'
import type { BrowserContext, Page, TestInfo } from '@playwright/test'
import { EventLoopDelayMonitor, type EventLoopDelaySummary } from './event-loop-diagnostics'
import {
  installPageCollector, type CollectorEventLabel, type PageCollectorEvent,
} from './page-stall-collector'
import {
  installScreenshotStageDiagnostics, resolveScreenshotServerPage,
  type ScreenshotStageEvent,
} from './screenshot-stage-diagnostics'

const STATE_LIMIT = 128
const ACTIVITY_LIMIT = 256
const GL_CALL_LIMIT = 64
const FAILURE_LIMIT = 32
const PHASE_LIMIT = 128
const TEMP_PAGE_LIMIT = 64
const SCHEDULER_INTERVAL_MS = 100
const REPORTING_GAP_MS = 50
const SLOW_GL_CALL_MS = 50
const HEARTBEAT_MS = 1_000
const GL_METHODS = new Set([
  'useProgram', 'deleteBuffer', 'deleteProgram', 'deleteShader', 'deleteTexture',
  'deleteFramebuffer', 'deleteRenderbuffer', 'deleteVertexArray', 'drawArrays',
  'drawElements', 'drawArraysInstanced', 'drawElementsInstanced', 'clear', 'getExtension',
  'deleteQuery', 'deleteSampler', 'deleteSync', 'deleteTransformFeedback',
  'drawRangeElements', 'loseContext',
])

// Page clocks mark creation; nodeAt marks binding arrival. Their delta can include
// page scheduling, binding queuing, IPC delivery, and Node scheduling.
type RetainedPageEvent = PageCollectorEvent & { nodeAt: number }
type GlCallTiming = {
  method: string
  contextOrdinal: number
  callCount: number
  pageAt: number
  documentOrdinal: number
  timeOrigin: number
  duration: number
  nodeAt: number
  stack: Array<{ functionName: string; asset: string; line: number; column: number }>
}

export type BrowserPageDiagnostics = {
  ordinal: number
  label: BrowserPageLabel
  nodeCreatedAt: number
  installDurationMs: number
  stateEvents: RetainedPageEvent[]
  activityEvents: RetainedPageEvent[]
  glCalls: GlCallTiming[]
  failures: DiagnosticFailure[]
  chromeWindowId?: number
  closedAt?: number
}

export type BrowserStallSummary = {
  nodeClock: { timeOrigin: number; capturedAt: number; eventLoop: EventLoopDelaySummary }
  pages: BrowserPageDiagnostics[]
  phases: DiagnosticPhaseEvent[]
  temporaryPages: TemporaryPageEvent[]
  screenshotStages: ScreenshotStageEvent[]
  failures: DiagnosticFailure[]
}

export type BrowserPageLabel = 'audio-primary' | 'audio-peer' | 'visibility-cover'
  | 'reconnect-primary' | 'reconnect-peer'
export type DiagnosticPhaseLabel = 'foreground' | 'axe' | 'width' | 'screenshot'
export type DiagnosticPhaseEvent = {
  label: DiagnosticPhaseLabel
  boundary: 'sent' | 'returned'
  pageOrdinal: number
  nodeAt: number
}
export type TemporaryPageEvent = {
  ordinal: number
  boundary: 'opened' | 'closed'
  nodeAt: number
}
type DiagnosticFailureStage = 'bindingSetup' | 'futureDocumentSetup' | 'currentDocumentSetup'
  | 'windowAssociation' | 'closeListenerSetup' | 'listenerCleanup' | 'observeAfterStop'
  | 'contextListenerSetup' | 'contextListenerCleanup' | 'temporaryPageListenerSetup'
  | 'eventLoopCleanup'
export type DiagnosticFailure = { stage: DiagnosticFailureStage; at: number }

function retain<T>(values: T[], value: T, limit: number): void {
  values.push(value)
  if (values.length > limit) values.shift()
}

function sanitizeStack(value: unknown): GlCallTiming['stack'] {
  if (!Array.isArray(value)) return []
  return value.slice(0, 8).flatMap((raw): GlCallTiming['stack'] => {
    if (typeof raw !== 'object' || raw === null) return []
    const candidate = raw as Record<string, unknown>
    const rawFunction = typeof candidate.functionName === 'string'
      ? candidate.functionName.slice(0, 256) : ''
    const functionName = /^[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*$/.test(rawFunction)
      ? rawFunction.slice(0, 64) : 'anonymous'
    const rawAsset = typeof candidate.asset === 'string'
      ? candidate.asset.slice(0, 2_048).split(/[?#]/, 1)[0]
        .replaceAll('\\', '/').split('/').at(-1) ?? '' : ''
    const asset = /^[A-Za-z0-9._-]+$/.test(rawAsset) ? rawAsset.slice(0, 64) : 'anonymous'
    const boundedInteger = (number: unknown): number => typeof number === 'number'
      && Number.isInteger(number) ? Math.min(1_000_000_000, Math.max(0, number)) : 0
    return [{
      functionName, asset,
      line: boundedInteger(candidate.line), column: boundedInteger(candidate.column),
    }]
  })
}

function finiteNumber(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

function positiveInteger(value: unknown): value is number {
  return finiteNumber(value) && Number.isInteger(value) && value >= 1
}

function glCall(candidate: Record<string, unknown>, nodeAt: number): GlCallTiming | undefined {
  if (candidate.label !== 'glCall'
    || typeof candidate.method !== 'string' || !GL_METHODS.has(candidate.method)
    || !positiveInteger(candidate.contextOrdinal) || !positiveInteger(candidate.callCount)
    || !finiteNumber(candidate.pageAt) || !positiveInteger(candidate.documentOrdinal)
    || !finiteNumber(candidate.timeOrigin) || !finiteNumber(candidate.duration)
    || candidate.duration < SLOW_GL_CALL_MS) return undefined
  return {
    method: candidate.method,
    contextOrdinal: candidate.contextOrdinal,
    callCount: candidate.callCount,
    pageAt: candidate.pageAt,
    documentOrdinal: candidate.documentOrdinal,
    timeOrigin: candidate.timeOrigin,
    duration: candidate.duration,
    nodeAt,
    stack: sanitizeStack(candidate.stack),
  }
}

function pageEvent(candidate: Record<string, unknown>, nodeAt: number): RetainedPageEvent | undefined {
  const allowed: CollectorEventLabel[] = [
    'initialized', 'visible', 'hidden', 'focused', 'blurred',
    'longTask', 'schedulingGap', 'bindingDuration', 'heartbeat', 'page-enter',
    'read-start', 'read-end',
  ]
  if (!allowed.includes(candidate.label as CollectorEventLabel)
    || !finiteNumber(candidate.pageAt) || !finiteNumber(candidate.documentOrdinal)
    || !finiteNumber(candidate.timeOrigin)) return undefined
  const event: RetainedPageEvent = {
    label: candidate.label as CollectorEventLabel,
    pageAt: candidate.pageAt,
    documentOrdinal: candidate.documentOrdinal,
    timeOrigin: candidate.timeOrigin,
    nodeAt,
  }
  if (finiteNumber(candidate.duration)) event.duration = Math.max(0, candidate.duration)
  return event
}

export class BrowserStallDiagnostics {
  private readonly eventLoop = new EventLoopDelayMonitor()
  private readonly pages: BrowserPageDiagnostics[] = []
  private readonly phases: DiagnosticPhaseEvent[] = []
  private readonly temporaryPages: TemporaryPageEvent[] = []
  private readonly screenshotStages: ScreenshotStageEvent[] = []
  private readonly failures: DiagnosticFailure[] = []
  private readonly observedPages = new Set<Page>()
  private readonly closeListeners: Array<{
    page: Page; listener: () => void; summary: BrowserPageDiagnostics
  }> = []
  private readonly contextListeners: Array<{
    context: BrowserContext; listener: (page: Page) => void
  }> = []
  private readonly observedContexts = new Set<BrowserContext>()
  private nextTemporaryPageOrdinal = 1
  private nextCaptureOrdinal = 1
  private readonly activeScreenshotCleanups = new Set<() => void>()
  private stopped = false

  async phase<T>(label: DiagnosticPhaseLabel, pageOrdinal: number,
    action: () => Promise<T>): Promise<T> {
    retain(this.phases, { label, boundary: 'sent', pageOrdinal, nodeAt: performance.now() }, PHASE_LIMIT)
    try {
      return await action()
    } finally {
      retain(this.phases, {
        label, boundary: 'returned', pageOrdinal, nodeAt: performance.now(),
      }, PHASE_LIMIT)
    }
  }

  async screenshot<T>(page: Page, pageOrdinal: number, action: () => Promise<T>): Promise<T> {
    return this.phase('screenshot', pageOrdinal, async () => {
      const captureOrdinal = this.nextCaptureOrdinal
      this.nextCaptureOrdinal += 1
      const cleanup = installScreenshotStageDiagnostics(
        resolveScreenshotServerPage(page), this.screenshotStages, pageOrdinal, captureOrdinal,
      )
      this.activeScreenshotCleanups.add(cleanup)
      try {
        return await action()
      } finally {
        cleanup()
        this.activeScreenshotCleanups.delete(cleanup)
      }
    })
  }

  private cleanupScreenshotDiagnostics(): void {
    for (const cleanup of this.activeScreenshotCleanups) cleanup()
    this.activeScreenshotCleanups.clear()
  }

  private observeContext(context: BrowserContext): void {
    if (this.observedContexts.has(context)) return
    this.observedContexts.add(context)
    const listener = (page: Page): void => {
      if (this.stopped || this.observedPages.has(page)) return
      const ordinal = this.nextTemporaryPageOrdinal
      this.nextTemporaryPageOrdinal += 1
      retain(this.temporaryPages, {
        ordinal, boundary: 'opened', nodeAt: performance.now(),
      }, TEMP_PAGE_LIMIT)
      try {
        page.once('close', () => {
          if (this.stopped) return
          retain(this.temporaryPages, {
            ordinal, boundary: 'closed', nodeAt: performance.now(),
          }, TEMP_PAGE_LIMIT)
        })
      } catch {
        retain(this.failures, {
          stage: 'temporaryPageListenerSetup', at: performance.now(),
        }, FAILURE_LIMIT)
      }
    }
    try {
      context.on('page', listener)
      this.contextListeners.push({ context, listener })
    } catch {
      retain(this.failures, { stage: 'contextListenerSetup', at: performance.now() }, FAILURE_LIMIT)
    }
  }

  async observePage(page: Page, label: BrowserPageLabel): Promise<void> {
    if (this.stopped) {
      retain(this.failures, { stage: 'observeAfterStop', at: performance.now() }, FAILURE_LIMIT)
      return
    }
    this.observedPages.add(page)
    this.observeContext(page.context())
    const startedAt = performance.now()
    const pageSummary: BrowserPageDiagnostics = {
      ordinal: this.pages.length + 1,
      label,
      nodeCreatedAt: startedAt,
      installDurationMs: 0,
      stateEvents: [],
      activityEvents: [],
      glCalls: [],
      failures: [],
    }
    this.pages.push(pageSummary)
    try {
      await page.exposeBinding('grenadiersStallDiagnostic', (_source, value: unknown) => {
        if (this.stopped || typeof value !== 'object' || value === null) return
        const candidate = value as Record<string, unknown>
        const arrival = performance.now()
        const retainedGlCall = glCall(candidate, arrival)
        if (retainedGlCall) {
          retain(pageSummary.glCalls, retainedGlCall, GL_CALL_LIMIT)
          return
        }
        const event = pageEvent(candidate, arrival)
        if (!event) return
        const target = ['initialized', 'visible', 'hidden', 'focused', 'blurred'].includes(event.label)
          ? pageSummary.stateEvents : pageSummary.activityEvents
        retain(target, event, target === pageSummary.stateEvents ? STATE_LIMIT : ACTIVITY_LIMIT)
      })
    } catch {
      retain(pageSummary.failures, { stage: 'bindingSetup', at: performance.now() }, FAILURE_LIMIT)
      pageSummary.installDurationMs = performance.now() - startedAt
      return
    }
    const collectorInput = {
      intervalMs: SCHEDULER_INTERVAL_MS, reportingGapMs: REPORTING_GAP_MS,
      slowGlCallMs: SLOW_GL_CALL_MS, heartbeatMs: HEARTBEAT_MS,
    }
    try { await page.addInitScript(installPageCollector, collectorInput) } catch {
      retain(pageSummary.failures, { stage: 'futureDocumentSetup', at: performance.now() }, FAILURE_LIMIT)
    }
    try { await page.evaluate(installPageCollector, collectorInput) } catch {
      retain(pageSummary.failures, { stage: 'currentDocumentSetup', at: performance.now() }, FAILURE_LIMIT)
    }
    try {
      const session = await page.context().newCDPSession(page)
      try {
        const associated = await session.send('Browser.getWindowForTarget')
        pageSummary.chromeWindowId = associated.windowId
      } finally { await session.detach() }
    } catch {
      retain(pageSummary.failures, { stage: 'windowAssociation', at: performance.now() }, FAILURE_LIMIT)
    }
    const closeListener = (): void => { pageSummary.closedAt = performance.now() }
    try {
      page.on('close', closeListener)
      this.closeListeners.push({ page, listener: closeListener, summary: pageSummary })
    } catch {
      retain(pageSummary.failures, { stage: 'closeListenerSetup', at: performance.now() }, FAILURE_LIMIT)
    }
    pageSummary.installDurationMs = performance.now() - startedAt
  }

  summary(): BrowserStallSummary {
    return {
      nodeClock: {
        timeOrigin: performance.timeOrigin,
        capturedAt: performance.now(),
        eventLoop: this.eventLoop.summary(),
      },
      pages: this.pages,
      phases: this.phases,
      temporaryPages: this.temporaryPages,
      screenshotStages: this.screenshotStages,
      failures: this.failures,
    }
  }

  async attach(testInfo: TestInfo, name: string): Promise<void> {
    this.cleanupScreenshotDiagnostics()
    try {
      await testInfo.attach(name, {
        contentType: 'application/json', body: JSON.stringify(this.summary()),
      })
    } catch {
      try {
        testInfo.annotations.push({
          type: 'browser-stall-diagnostics', description: 'attachment-failed',
        })
      } catch { /* Optional diagnostic annotation cannot replace the scenario result. */ }
    }
  }

  stop(): void {
    if (this.stopped) return
    this.stopped = true
    this.cleanupScreenshotDiagnostics()
    for (const { page, listener, summary } of this.closeListeners) {
      try { page.off('close', listener) } catch {
        retain(summary.failures, { stage: 'listenerCleanup', at: performance.now() }, FAILURE_LIMIT)
      }
    }
    this.closeListeners.length = 0
    for (const { context, listener } of this.contextListeners) {
      try { context.off('page', listener) } catch {
        retain(this.failures, { stage: 'contextListenerCleanup', at: performance.now() }, FAILURE_LIMIT)
      }
    }
    this.contextListeners.length = 0
    try { this.eventLoop.stop() } catch {
      retain(this.failures, { stage: 'eventLoopCleanup', at: performance.now() }, FAILURE_LIMIT)
    }
  }
}

export function startBrowserStallDiagnostics(): BrowserStallDiagnostics {
  return new BrowserStallDiagnostics()
}
