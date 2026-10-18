import { performance } from 'node:perf_hooks'
import type { Page, TestInfo } from '@playwright/test'
import type { GameServerMessage } from '../../../web/src/game/v7/schema'
import { CaptureBrowserTrace, type CaptureBrowserTraceSummary } from './capture-browser-trace'
import {
  preparePageRenderCounters, removeRenderCounters, resetRenderCounters, stopAndReadRenderCounters,
  type PageRenderCounters,
} from './page-render-counters'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type CanvasProfile = {
  pageOrdinal: number
  backingWidth: number
  backingHeight: number
  cssWidth: number
  cssHeight: number
  devicePixelRatio: number
  visibility: 'visible' | 'hidden' | 'prerender' | 'unloaded' | 'unknown'
}
type WorkloadCounts = {
  terrain: number
  blocks: number
  actors: number
  bombs: number
  flames: number
  visiblePickups: number
}
type ProfileFailure = 'counter-install' | 'profile-start' | 'profile-stop'
export type CaptureWorkloadSummary = {
  enabled: boolean
  counterSemantics: 'cpu-submissions-and-callbacks-not-gpu-completion-or-frames-source-span-upper-bounds'
  automaticFailureScreenshots: 'disabled-when-profile-enabled'
  window?: { startedAt: number; endedAt: number; outcome: 'success' | 'error' }
  workload?: WorkloadCounts
  canvases: CanvasProfile[]
  renderCounters: Array<{ pageOrdinal: number; counters: PageRenderCounters }>
  browserTrace: CaptureBrowserTraceSummary
  failures: ProfileFailure[]
}

function emptyTrace(): CaptureBrowserTraceSummary {
  return {
    eventPresenceIsNonExhaustive: true, available: false,
    unavailableCategories: [], events: [], droppedEvents: 0,
    eventAggregates: [], truncated: false, traceCompleted: false,
    cpuBeforeStatus: 'not-attempted', traceStartStatus: 'not-attempted',
    clockMarkerStatus: 'not-attempted', traceEndStatus: 'not-attempted',
    cpuAfterStatus: 'not-attempted', detachStatus: 'not-attempted',
    cpuBefore: [], cpuAfter: [],
  }
}

export class CaptureWorkloadProfile {
  readonly summary: CaptureWorkloadSummary
  readonly enabled: boolean
  private readonly trace = new CaptureBrowserTrace()
  private pages: Page[] = []
  private ended = false

  constructor(enabled: boolean) {
    this.enabled = enabled
    this.summary = {
      enabled,
      counterSemantics: 'cpu-submissions-and-callbacks-not-gpu-completion-or-frames-source-span-upper-bounds',
      automaticFailureScreenshots: 'disabled-when-profile-enabled',
      canvases: [], renderCounters: [], browserTrace: emptyTrace(), failures: [],
    }
  }

  async install(pages: Page[]): Promise<void> {
    if (!this.enabled) return
    this.pages = pages
    const installed = await Promise.allSettled(pages.map(preparePageRenderCounters))
    if (installed.some((result) => result.status === 'rejected'))
      this.summary.failures.push('counter-install')
  }

  async begin(primary: Page, snapshot: Snapshot): Promise<void> {
    if (!this.enabled || this.summary.window) return
    const state = snapshot.state
    this.summary.workload = {
      terrain: state.arena.terrain.length,
      blocks: state.arena.blocks.length,
      actors: state.actors.length,
      bombs: state.bombs.length,
      flames: state.flames.length,
      visiblePickups: state.pickups.length,
    }
    this.summary.window = { startedAt: performance.now(), endedAt: 0, outcome: 'error' }
    try {
      await Promise.all(this.pages.map(async (page, index) => {
        await page.evaluate(resetRenderCounters)
        const canvas = await page.locator('.game-arena__canvas canvas').evaluate((element) => {
          const value = element as HTMLCanvasElement
          const rect = value.getBoundingClientRect()
          const allowed = ['visible', 'hidden', 'prerender', 'unloaded']
          return {
            backingWidth: value.width, backingHeight: value.height,
            cssWidth: rect.width, cssHeight: rect.height, devicePixelRatio,
            visibility: allowed.includes(document.visibilityState)
              ? document.visibilityState : 'unknown',
          }
        })
        this.summary.canvases.push({ pageOrdinal: index + 1, ...canvas } as CanvasProfile)
      }))
      await this.trace.start(primary)
      this.summary.browserTrace = this.trace.summary
    } catch {
      this.summary.failures.push('profile-start')
    }
  }

  async end(outcome: 'success' | 'error'): Promise<void> {
    if (!this.enabled || this.ended || !this.summary.window) return
    this.ended = true
    this.summary.window.endedAt = performance.now()
    this.summary.window.outcome = outcome
    const counters = await Promise.allSettled(this.pages.map(
      (page) => page.evaluate(stopAndReadRenderCounters),
    ))
    this.summary.renderCounters = counters.flatMap((result, index) => result.status === 'fulfilled'
      ? [{ pageOrdinal: index + 1, counters: result.value }] : [])
    if (counters.some((result) => result.status === 'rejected'))
      this.summary.failures.push('profile-stop')
    try {
      await this.trace.stop()
    } catch {
      this.summary.failures.push('profile-stop')
    }
    this.summary.browserTrace = this.trace.summary
  }

  async cleanup(): Promise<void> {
    if (!this.enabled) return
    try { await this.end('error') } catch { /* Error priority belongs to the scenario. */ }
    await Promise.allSettled(this.pages.map((page) => page.evaluate(removeRenderCounters)))
  }

  async attach(testInfo: TestInfo): Promise<void> {
    if (!this.enabled) return
    try {
      await testInfo.attach('capture-workload-profile', {
        contentType: 'application/json', body: JSON.stringify(this.summary),
      })
    } catch { /* Optional profiler attachment cannot replace the scenario result. */ }
  }
}

export function startCaptureWorkloadProfile(): CaptureWorkloadProfile {
  return new CaptureWorkloadProfile(process.env.E2E_CAPTURE_PROFILE === '1')
}
