import { createRequire } from 'node:module'
import { performance } from 'node:perf_hooks'
import type { CDPSession, Page } from '@playwright/test'

const require = createRequire(import.meta.url)
const REQUIRED_CATEGORIES = ['cc', 'viz', 'gpu', 'devtools'] as const
const PROCESS_TYPES = new Set(['browser', 'renderer', 'GPU', 'utility'])
const EVENT_NAMES = [
  'EncodeBitmapAsPngSlow', 'EncodeBitmapAsPngFast', 'EncodeBitmapAsJpeg',
  'EncodeBitmapAsWebp', 'Surface::RequestCopyOfOutput',
  'Surface::RequestCopyOfOutputOnRootRenderPass',
  'Surface::RequestCopyOfOurpurRenderPassId', 'Display::DrawAndSwap',
  'Graphics.Pipeline', 'Graphics.Pipeline.DrawAndSwap',
] as const
const EVENT_NAME_SET = new Set<string>(EVENT_NAMES)
const PHASES = new Set(['B', 'E', 'X', 'I', 'i', 'b', 'e', 's', 't', 'f', 'n'])
const EVENT_LIMIT = 512
const SAMPLE_LIMIT = 8
const CLOCK_SYNC_ID = 'grenadiers-capture-profile'

type ApprovedEventName = typeof EVENT_NAMES[number]
type ApprovedCategory = typeof REQUIRED_CATEGORIES[number]

type SanitizedTraceEvent = {
  name: ApprovedEventName
  timestamp: number
  duration?: number
  processId: number
  threadId: number
  category: ApprovedCategory
  phase: string
  flowId?: number
  flowId2?: number
  unsupportedFlowIds?: true
}
type ProcessCpu = { type: string; processId: number; cpuTime: number }
type TraceStatus = 'not-attempted' | 'success' | 'error'
type TraceEndStatus = 'not-attempted' | 'completed' | 'command-error' | 'completion-timeout'
type EventAggregate = {
  name: ApprovedEventName
  count: number
  totalDuration: number
  firstTimestamp?: number
  lastTimestamp?: number
  maxDuration?: number
  unsupportedFlowIdEvents: number
  samples: SanitizedTraceEvent[]
  samplesTruncated: boolean
}
export type CaptureBrowserTraceSummary = {
  eventPresenceIsNonExhaustive: true
  available: boolean
  unavailableReason?: 'playwright-version' | 'browser-version' | 'categories' | 'protocol'
  unavailableCategories: string[]
  nodeClockBeforeMarker?: number
  nodeClockAfterMarker?: number
  traceClockTimestamp?: number
  events: SanitizedTraceEvent[]
  eventAggregates: EventAggregate[]
  droppedEvents: number
  truncated: boolean
  traceCompleted: boolean
  traceDataLoss?: boolean
  cpuBeforeStatus: TraceStatus
  traceStartStatus: TraceStatus
  clockMarkerStatus: TraceStatus
  traceEndStatus: TraceEndStatus
  cpuAfterStatus: TraceStatus
  detachStatus: TraceStatus
  cpuBefore: ProcessCpu[]
  cpuAfter: ProcessCpu[]
}

function finite(value: unknown): value is number {
  return typeof value === 'number' && Number.isFinite(value)
}

function sanitizeProcesses(value: unknown): ProcessCpu[] {
  if (!Array.isArray(value)) return []
  return value.flatMap((raw): ProcessCpu[] => {
    if (!raw || typeof raw !== 'object') return []
    const item = raw as Record<string, unknown>
    if (typeof item.type !== 'string' || !PROCESS_TYPES.has(item.type)
      || !finite(item.id) || !finite(item.cpuTime)) return []
    return [{ type: item.type, processId: item.id, cpuTime: item.cpuTime }]
  }).slice(0, 64)
}

function clockSyncTimestamp(raw: unknown): number | undefined {
  if (!raw || typeof raw !== 'object') return undefined
  const value = raw as Record<string, unknown>
  if (value.name !== 'clock_sync' || !finite(value.ts)
    || !value.args || typeof value.args !== 'object') return undefined
  return (value.args as Record<string, unknown>).sync_id === CLOCK_SYNC_ID ? value.ts : undefined
}

function boundedFlowId(value: unknown): number | undefined {
  return finite(value) && Number.isSafeInteger(value) && value >= 0 ? value : undefined
}

function hasUnsupportedFlowIds(value: Record<string, unknown>): boolean {
  return [value.id, value.id2].some(
    (candidate) => candidate !== undefined && boundedFlowId(candidate) === undefined,
  )
}

function sanitizeEvent(raw: unknown): SanitizedTraceEvent | undefined {
  if (!raw || typeof raw !== 'object') return undefined
  const value = raw as Record<string, unknown>
  const name = typeof value.name === 'string' && EVENT_NAME_SET.has(value.name)
    ? value.name as ApprovedEventName : undefined
  const categories = typeof value.cat === 'string' ? value.cat.split(',') : []
  const category = REQUIRED_CATEGORIES.find((candidate) => categories.includes(candidate))
  const phase = typeof value.ph === 'string' && PHASES.has(value.ph) ? value.ph : undefined
  if (!name || !category || !phase || !finite(value.ts) || !finite(value.pid) || !finite(value.tid))
    return undefined
  const event: SanitizedTraceEvent = {
    name, timestamp: value.ts, processId: value.pid, threadId: value.tid, category, phase,
  }
  if (finite(value.dur)) event.duration = Math.max(0, value.dur)
  const flowId = boundedFlowId(value.id)
  const flowId2 = boundedFlowId(value.id2)
  if (flowId !== undefined) event.flowId = flowId
  if (flowId2 !== undefined) event.flowId2 = flowId2
  if (hasUnsupportedFlowIds(value)) event.unsupportedFlowIds = true
  return event
}

export class CaptureBrowserTrace {
  readonly summary: CaptureBrowserTraceSummary = {
    eventPresenceIsNonExhaustive: true, available: false,
    unavailableCategories: [], events: [], droppedEvents: 0,
    eventAggregates: EVENT_NAMES.map((name) => ({
      name, count: 0, totalDuration: 0, unsupportedFlowIdEvents: 0,
      samples: [], samplesTruncated: false,
    })),
    truncated: false, traceCompleted: false,
    cpuBeforeStatus: 'not-attempted', traceStartStatus: 'not-attempted',
    clockMarkerStatus: 'not-attempted', traceEndStatus: 'not-attempted',
    cpuAfterStatus: 'not-attempted', detachStatus: 'not-attempted',
    cpuBefore: [], cpuAfter: [],
  }
  private session?: CDPSession
  private tracing = false
  private accepting = false
  private completion?: Promise<void>
  private complete!: () => void
  private readonly dataListener = (payload: { value: unknown[] }): void => {
    if (!this.accepting) return
    for (const raw of payload.value) this.retain(raw)
  }
  private readonly completeListener = (payload: { dataLossOccurred: boolean }): void => {
    if (!this.accepting) return
    this.summary.traceCompleted = true
    this.summary.traceDataLoss = payload.dataLossOccurred
    this.complete()
  }

  async start(page: Page): Promise<void> {
    const core = require('playwright-core/package.json') as { version?: unknown }
    if (core.version !== '1.63.0') {
      this.summary.unavailableReason = 'playwright-version'
      return
    }
    const browser = page.context().browser()
    if (browser?.version() !== '153.0.8010.12') {
      this.summary.unavailableReason = 'browser-version'
      return
    }
    try {
      this.session = await browser.newBrowserCDPSession()
      const categories = await this.session.send('Tracing.getCategories')
      const available = new Set(categories.categories)
      this.summary.unavailableCategories = REQUIRED_CATEGORIES.filter((value) => !available.has(value))
      if (this.summary.unavailableCategories.length) {
        this.summary.unavailableReason = 'categories'
        return
      }
      try {
        const processes = await this.session.send('SystemInfo.getProcessInfo')
        this.summary.cpuBefore = sanitizeProcesses(processes.processInfo)
        this.summary.cpuBeforeStatus = 'success'
      } catch { this.summary.cpuBeforeStatus = 'error' }
      this.completion = new Promise<void>((resolve) => { this.complete = resolve })
      this.accepting = true
      this.session.on('Tracing.dataCollected', this.dataListener)
      this.session.once('Tracing.tracingComplete', this.completeListener)
      await this.session.send('Tracing.start', {
        transferMode: 'ReportEvents', tracingBackend: 'chrome',
        traceConfig: {
          recordMode: 'recordUntilFull', traceBufferSizeInKb: 8_192,
          enableArgumentFilter: true, includedCategories: [...REQUIRED_CATEGORIES],
          excludedCategories: [
            'disabled-by-default-devtools.screenshot', 'memory-infra',
            'disabled-by-default-v8.cpu_profiler', 'netlog', 'network',
          ],
        },
      })
      this.tracing = true
      this.summary.available = true
      this.summary.traceStartStatus = 'success'
      this.summary.nodeClockBeforeMarker = performance.now()
      try {
        await this.session.send('Tracing.recordClockSyncMarker', { syncId: CLOCK_SYNC_ID })
        this.summary.clockMarkerStatus = 'success'
      } catch { this.summary.clockMarkerStatus = 'error' }
      finally { this.summary.nodeClockAfterMarker = performance.now() }
    } catch {
      this.summary.available = false
      this.summary.unavailableReason = 'protocol'
      if (this.summary.traceStartStatus === 'not-attempted') this.summary.traceStartStatus = 'error'
    }
  }

  private retain(raw: unknown): void {
    const clockTimestamp = clockSyncTimestamp(raw)
    if (clockTimestamp !== undefined) {
      if (this.summary.traceClockTimestamp === undefined)
        this.summary.traceClockTimestamp = clockTimestamp
      return
    }
    const event = sanitizeEvent(raw)
    if (!event) {
      this.summary.droppedEvents += 1
      return
    }
    const aggregate = this.summary.eventAggregates.find(
      (candidate) => candidate.name === event.name,
    )!
    aggregate.firstTimestamp = Math.min(aggregate.firstTimestamp ?? event.timestamp, event.timestamp)
    aggregate.lastTimestamp = Math.max(aggregate.lastTimestamp ?? event.timestamp, event.timestamp)
    if (event.duration !== undefined)
      aggregate.maxDuration = Math.max(aggregate.maxDuration ?? event.duration, event.duration)
    if (event.unsupportedFlowIds)
      aggregate.unsupportedFlowIdEvents = Math.min(
        Number.MAX_SAFE_INTEGER, aggregate.unsupportedFlowIdEvents + 1,
      )
    aggregate.count = Math.min(Number.MAX_SAFE_INTEGER, aggregate.count + 1)
    aggregate.totalDuration = Math.min(
      Number.MAX_SAFE_INTEGER, aggregate.totalDuration + (event.duration ?? 0),
    )
    aggregate.samples.push(event)
    if (aggregate.samples.length > SAMPLE_LIMIT) {
      aggregate.samples.shift()
      aggregate.samplesTruncated = true
    }
    this.summary.events.push(event)
    if (this.summary.events.length > EVENT_LIMIT) {
      this.summary.events.shift()
      this.summary.truncated = true
    }
  }

  async stop(): Promise<void> {
    const session = this.session
    if (!session) return
    try {
      if (this.tracing) {
        this.tracing = false
        try {
          await session.send('Tracing.end')
          const completed = await Promise.race([
            this.completion?.then(() => true),
            new Promise<false>((resolve) => setTimeout(() => resolve(false), 2_000)),
          ])
          this.summary.traceEndStatus = completed ? 'completed' : 'completion-timeout'
        } catch { this.summary.traceEndStatus = 'command-error' }
      }
      if (this.summary.available) {
        try {
          const processes = await session.send('SystemInfo.getProcessInfo')
          this.summary.cpuAfter = sanitizeProcesses(processes.processInfo)
          this.summary.cpuAfterStatus = 'success'
        } catch { this.summary.cpuAfterStatus = 'error' }
      }
    } catch { /* Optional profiler cannot replace the scenario result. */ }
    this.accepting = false
    try {
      session.removeListener('Tracing.dataCollected', this.dataListener)
      session.removeListener('Tracing.tracingComplete', this.completeListener)
    } catch { /* A closed dedicated session has already discarded its listeners. */ }
    try {
      await session.detach()
      this.summary.detachStatus = 'success'
    } catch { this.summary.detachStatus = 'error' }
    this.session = undefined
  }
}

export function sanitizeTraceEventsForTest(values: Array<Record<string, unknown>>): CaptureBrowserTraceSummary {
  const trace = new CaptureBrowserTrace()
  for (const value of values) trace['retain'](value)
  return trace.summary
}
