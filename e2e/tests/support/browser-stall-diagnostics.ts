import { performance } from 'node:perf_hooks'
import type { Page, TestInfo } from '@playwright/test'
import { EventLoopDelayMonitor, type EventLoopDelaySummary } from './event-loop-diagnostics'

const STATE_LIMIT = 128
const ACTIVITY_LIMIT = 256
const FAILURE_LIMIT = 32
const SCHEDULER_INTERVAL_MS = 100
const REPORTING_GAP_MS = 50

type PageEventLabel = 'initialized' | 'visible' | 'hidden' | 'focused' | 'blurred'
  | 'longTask' | 'schedulingGap' | 'bindingDuration'

type PageEvent = {
  label: PageEventLabel
  pageAt: number
  documentOrdinal: number
  timeOrigin: number
  duration?: number
}

type RetainedPageEvent = PageEvent & { nodeAt: number }

export type BrowserPageDiagnostics = {
  ordinal: number
  label: BrowserPageLabel
  nodeCreatedAt: number
  installDurationMs: number
  stateEvents: RetainedPageEvent[]
  activityEvents: RetainedPageEvent[]
  failures: DiagnosticFailure[]
  chromeWindowId?: number
  closedAt?: number
}

export type BrowserStallSummary = {
  nodeClock: { timeOrigin: number; capturedAt: number; eventLoop: EventLoopDelaySummary }
  pages: BrowserPageDiagnostics[]
  failures: DiagnosticFailure[]
}

type DiagnosticBinding = (event: PageEvent) => Promise<void>
type DiagnosticWindow = Window & {
  grenadiersStallDiagnostic?: DiagnosticBinding
  grenadiersStallCollectorInstalled?: true
}
export type BrowserPageLabel = 'audio-primary' | 'audio-peer' | 'visibility-cover'
  | 'reconnect-primary' | 'reconnect-peer'
type DiagnosticFailureStage = 'bindingSetup' | 'futureDocumentSetup' | 'currentDocumentSetup'
  | 'windowAssociation' | 'closeListenerSetup' | 'listenerCleanup' | 'observeAfterStop'
  | 'eventLoopCleanup'
export type DiagnosticFailure = { stage: DiagnosticFailureStage; at: number }

function retain<T>(values: T[], value: T, limit: number): void {
  values.push(value)
  if (values.length > limit) values.shift()
}

function installPageCollector(input: { intervalMs: number; reportingGapMs: number }): void {
  const observed = window as DiagnosticWindow
  const binding = observed.grenadiersStallDiagnostic
  if (!binding || observed.grenadiersStallCollectorInstalled) return
  observed.grenadiersStallCollectorInstalled = true
  let documentOrdinal = 0
  let observedRoot: Element | null = null
  const send = (event: Omit<PageEvent, 'documentOrdinal' | 'timeOrigin'>): void => {
    const started = performance.now()
    const contextual = {
      ...event, documentOrdinal, timeOrigin: performance.timeOrigin,
    }
    void binding(contextual).then(() => binding({
      label: 'bindingDuration', pageAt: performance.now(),
      documentOrdinal, timeOrigin: performance.timeOrigin,
      duration: performance.now() - started,
    })).catch(() => undefined)
  }
  const state = (): void => send({
    label: document.visibilityState === 'visible' ? 'visible' : 'hidden',
    pageAt: performance.now(),
  })
  const initializeDocument = (): void => {
    documentOrdinal += 1
    observedRoot = document.documentElement
    send({ label: 'initialized', pageAt: performance.now() })
    state()
    document.addEventListener('visibilitychange', state)
  }
  initializeDocument()
  send({ label: document.hasFocus() ? 'focused' : 'blurred', pageAt: performance.now() })
  addEventListener('focus', () => send({ label: 'focused', pageAt: performance.now() }))
  addEventListener('blur', () => send({ label: 'blurred', pageAt: performance.now() }))
  if ('PerformanceObserver' in window) {
    try {
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          send({ label: 'longTask', pageAt: entry.startTime, duration: entry.duration })
        }
      }).observe({ type: 'longtask', buffered: true })
    } catch { /* Long-task observation is optional in this browser. */ }
  }
  let expected = performance.now() + input.intervalMs
  setInterval(() => {
    const now = performance.now()
    if (observedRoot !== document.documentElement) initializeDocument()
    const gap = Math.max(0, now - expected)
    if (gap >= input.reportingGapMs) {
      send({ label: 'schedulingGap', pageAt: now, duration: gap })
    }
    expected = now + input.intervalMs
  }, input.intervalMs)
}

export class BrowserStallDiagnostics {
  private readonly eventLoop = new EventLoopDelayMonitor()
  private readonly pages: BrowserPageDiagnostics[] = []
  private readonly failures: DiagnosticFailure[] = []
  private readonly closeListeners: Array<{
    page: Page; listener: () => void; summary: BrowserPageDiagnostics
  }> = []
  private stopped = false

  async observePage(page: Page, label: BrowserPageLabel): Promise<void> {
    if (this.stopped) {
      retain(this.failures, { stage: 'observeAfterStop', at: performance.now() }, FAILURE_LIMIT)
      return
    }
    const startedAt = performance.now()
    const pageSummary: BrowserPageDiagnostics = {
      ordinal: this.pages.length + 1,
      label,
      nodeCreatedAt: startedAt,
      installDurationMs: 0,
      stateEvents: [],
      activityEvents: [],
      failures: [],
    }
    this.pages.push(pageSummary)
    try {
      await page.exposeBinding('grenadiersStallDiagnostic', (_source, value: unknown) => {
        if (this.stopped || typeof value !== 'object' || value === null) return
        const candidate = value as Record<string, unknown>
        const allowed: PageEventLabel[] = [
          'initialized', 'visible', 'hidden', 'focused', 'blurred',
          'longTask', 'schedulingGap', 'bindingDuration',
        ]
        if (!allowed.includes(candidate.label as PageEventLabel)
          || typeof candidate.pageAt !== 'number' || !Number.isFinite(candidate.pageAt)
          || typeof candidate.documentOrdinal !== 'number' || !Number.isFinite(candidate.documentOrdinal)
          || typeof candidate.timeOrigin !== 'number' || !Number.isFinite(candidate.timeOrigin)) return
        const event: RetainedPageEvent = {
          label: candidate.label as PageEventLabel,
          pageAt: candidate.pageAt,
          documentOrdinal: candidate.documentOrdinal,
          timeOrigin: candidate.timeOrigin,
          nodeAt: performance.now(),
        }
        if (typeof candidate.duration === 'number' && Number.isFinite(candidate.duration)) {
          event.duration = Math.max(0, candidate.duration)
        }
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
      failures: this.failures,
    }
  }

  async attach(testInfo: TestInfo, name: string): Promise<void> {
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
    for (const { page, listener, summary } of this.closeListeners) {
      try { page.off('close', listener) } catch {
        retain(summary.failures, { stage: 'listenerCleanup', at: performance.now() }, FAILURE_LIMIT)
      }
    }
    this.closeListeners.length = 0
    try { this.eventLoop.stop() } catch {
      retain(this.failures, { stage: 'eventLoopCleanup', at: performance.now() }, FAILURE_LIMIT)
    }
  }
}

export function startBrowserStallDiagnostics(): BrowserStallDiagnostics {
  return new BrowserStallDiagnostics()
}
