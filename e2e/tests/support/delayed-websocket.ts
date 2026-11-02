import { performance } from 'node:perf_hooks'
import type { Page, WebSocketRoute } from '@playwright/test'
import type { Direction, GameServerMessage, Position } from '../../../web/src/game/v20/schema'

export type DelayScheduler = {
  now: () => number
  setTimer: (callback: () => void, delayMs: number) => unknown
  clearTimer: (timer: unknown) => void
}

export type DelayQueueStats = {
  delivered: number
  canceled: number
  peakFrames: number
  peakBytes: number
}

export type DelayTiming = {
  enqueueAt: number
  dueAt: number
  dispatchAt: number
  timerLatenessMs: number
}

export class QueueOverflowError extends Error {}

type Pending<T> = { value: T; bytes: number; enqueueAt: number; dueAt: number }
type DelayQueueOptions = { scheduler?: DelayScheduler; maxFrames?: number; maxBytes?: number }

export class DelayedFifo<T> {
  private readonly pending: Pending<T>[] = []
  private timer: unknown
  private bytes = 0
  private jitterIndex = 0
  private previousDue = Number.NEGATIVE_INFINITY
  private closed = false
  private readonly baseMs: number
  private readonly jitterMs: readonly number[]
  private readonly deliver: (value: T, timing: DelayTiming) => void
  private readonly scheduler: DelayScheduler
  private readonly maxFrames: number
  private readonly maxBytes: number
  readonly stats: DelayQueueStats = { delivered: 0, canceled: 0, peakFrames: 0, peakBytes: 0 }

  constructor(
    baseMs: number,
    jitterMs: readonly number[],
    deliver: (value: T, timing: DelayTiming) => void,
    options: DelayQueueOptions = {},
  ) {
    const { scheduler = realScheduler, maxFrames = 64, maxBytes = 1_048_576 } = options
    if (!Number.isFinite(baseMs) || jitterMs.length === 0
      || jitterMs.some((value) => !Number.isFinite(value) || baseMs + value < 0)) {
      throw new Error('Delay profile must contain finite nonnegative delays')
    }
    if (!Number.isSafeInteger(maxFrames) || maxFrames < 1
      || !Number.isSafeInteger(maxBytes) || maxBytes < 1) {
      throw new Error('Delay queue bounds must be positive safe integers')
    }
    this.baseMs = baseMs
    this.jitterMs = jitterMs
    this.deliver = deliver
    this.scheduler = scheduler
    this.maxFrames = maxFrames
    this.maxBytes = maxBytes
  }

  push(value: T, bytes: number): Pick<DelayTiming, 'enqueueAt' | 'dueAt'> {
    if (this.closed) throw new Error('Delay queue is closed')
    if (!Number.isSafeInteger(bytes) || bytes < 0 || this.pending.length >= this.maxFrames
      || this.bytes + bytes > this.maxBytes) {
      throw new QueueOverflowError('Delayed WebSocket queue exceeded its bounded capacity')
    }
    const jitter = this.jitterMs[this.jitterIndex % this.jitterMs.length]
    this.jitterIndex += 1
    const enqueueAt = this.scheduler.now()
    const dueAt = Math.max(this.previousDue, enqueueAt + this.baseMs + jitter)
    this.previousDue = dueAt
    this.pending.push({ value, bytes, enqueueAt, dueAt })
    this.bytes += bytes
    this.stats.peakFrames = Math.max(this.stats.peakFrames, this.pending.length)
    this.stats.peakBytes = Math.max(this.stats.peakBytes, this.bytes)
    this.schedule()
    return { enqueueAt, dueAt }
  }

  cancel(): void {
    if (this.closed) return
    this.closed = true
    if (this.timer !== undefined) this.scheduler.clearTimer(this.timer)
    this.timer = undefined
    this.stats.canceled += this.pending.length
    this.pending.length = 0
    this.bytes = 0
  }

  private schedule(): void {
    if (this.timer !== undefined || this.pending.length === 0) return
    const wait = Math.max(0, this.pending[0].dueAt - this.scheduler.now())
    this.timer = this.scheduler.setTimer(() => this.drain(), wait)
  }

  private drain(): void {
    this.timer = undefined
    const now = this.scheduler.now()
    while (this.pending[0]?.dueAt <= now) {
      const next = this.pending.shift()!
      this.bytes -= next.bytes
      this.deliver(next.value, {
        enqueueAt: next.enqueueAt,
        dueAt: next.dueAt,
        dispatchAt: now,
        timerLatenessMs: Math.max(0, now - next.dueAt),
      })
      this.stats.delivered += 1
    }
    this.schedule()
  }
}

const realScheduler: DelayScheduler = {
  now: () => performance.now(),
  setTimer: (callback, delayMs) => setTimeout(callback, delayMs),
  clearTimer: (timer) => clearTimeout(timer as ReturnType<typeof setTimeout>),
}

export type RoutedFrameSummary = {
  endpoint: string
  direction: 'clientToServer' | 'serverToClient'
  at: number
  bytes: number
  type: string
  movement?: Direction | null
  actionId?: number
  movementSequence?: number
  snapshot?: SanitizedSnapshot
  enqueueAt?: number
  dueAt?: number
  dispatchAt?: number
  timerLatenessMs?: number
}

export type SanitizedSnapshot = {
  revision: number
  tick: number
  roundId: number
  position: Position
  bombIds: number[]
}

export type DelayBridgeSummary = {
  endpoint: string
  closed: boolean
  clientToServer: DelayQueueStats
  serverToClient: DelayQueueStats
}

export type DelayBridgeObserver = {
  frame: (summary: RoutedFrameSummary) => void
  forwarded: (summary: RoutedFrameSummary) => void
  bridge: (summary: DelayBridgeSummary) => void
  failure: (message: string) => void
}

export type DelayBridgeHandle = {
  summary: () => DelayBridgeSummary | undefined
  close: () => void
}

type DelayProfile = { baseMs: number; jitterMs: readonly number[] }
type DelayedMessage = { message: string | Buffer; summary: RoutedFrameSummary }

function frameSize(message: string | Buffer): number {
  return typeof message === 'string' ? Buffer.byteLength(message) : message.byteLength
}

function frameSummary(
  endpoint: string,
  direction: RoutedFrameSummary['direction'],
  message: string | Buffer,
): RoutedFrameSummary {
  const summary: RoutedFrameSummary = {
    endpoint, direction, at: performance.now(), bytes: frameSize(message), type: 'binary',
  }
  if (typeof message !== 'string') return summary
  const type = /"type"\s*:\s*"([^"]+)"/.exec(message)?.[1]
  summary.type = type ?? 'text'
  try {
    const parsed = JSON.parse(message) as GameServerMessage & {
      direction?: unknown
      actionId?: unknown
      sequence?: unknown
    }
    if (direction === 'serverToClient' && parsed.type === 'snapshot') {
      const actor = parsed.state.actors.find((value) => value.seat === 'one')
      if (actor?.type === 'visible') summary.snapshot = {
        revision: parsed.room.revision, tick: parsed.room.tick, roundId: parsed.room.roundId,
        position: actor.position, bombIds: parsed.state.bombs.map((bomb) => bomb.id),
      }
      return summary
    }
    if (summary.type === 'movement'
      && (parsed.direction === null || ['north', 'east', 'south', 'west'].includes(String(parsed.direction)))) {
      summary.movement = parsed.direction as Direction | null
      if (Number.isSafeInteger(parsed.sequence)) summary.movementSequence = parsed.sequence as number
    }
    if (summary.type === 'manualAction' && Number.isSafeInteger(parsed.actionId)) {
      summary.actionId = parsed.actionId as number
    }
  } catch { /* The original opaque text is still forwarded unchanged. */ }
  return summary
}

function closeOptions(code: number | undefined, reason: string | undefined) {
  return code === undefined && reason === undefined ? undefined : { code, reason }
}

export async function installDelayedWebSocket(
  page: Page,
  endpoint: string,
  profile: DelayProfile,
  observer: DelayBridgeObserver,
): Promise<DelayBridgeHandle> {
  let current: (() => DelayBridgeSummary) | undefined
  let closeCurrent = (): void => undefined
  await page.routeWebSocket((url) => url.pathname.endsWith('/game/ws'), (client) => {
    const server = client.connectToServer()
    let closed = false
    const forward = (target: WebSocketRoute, delayed: DelayedMessage, timing: DelayTiming): void => {
      const at = performance.now()
      observer.forwarded({
        ...delayed.summary, ...timing, at, dispatchAt: at,
        timerLatenessMs: Math.max(0, at - timing.dueAt),
      })
      target.send(delayed.message)
    }
    const clientToServer = new DelayedFifo<DelayedMessage>(
      profile.baseMs, profile.jitterMs, (delayed, timing) => forward(server, delayed, timing),
    )
    const serverToClient = new DelayedFifo<DelayedMessage>(
      profile.baseMs, profile.jitterMs, (delayed, timing) => forward(client, delayed, timing),
    )
    const summary = (): DelayBridgeSummary => ({
      endpoint, closed, clientToServer: { ...clientToServer.stats }, serverToClient: { ...serverToClient.stats },
    })
    current = summary
    const finish = (): void => observer.bridge(summary())
    const shutdown = (source: WebSocketRoute, target: WebSocketRoute, code?: number, reason?: string): void => {
      if (closed) return
      closed = true
      clientToServer.cancel()
      serverToClient.cancel()
      finish()
      void target.close(closeOptions(code, reason)).catch(() => undefined)
      void source.close(closeOptions(code, reason)).catch(() => undefined)
    }
    closeCurrent = () => shutdown(client, server, 1000, 'Latency fixture complete')
    const enqueue = (
      direction: RoutedFrameSummary['direction'],
      queue: DelayedFifo<DelayedMessage>,
      message: string | Buffer,
    ): void => {
      const summary = frameSummary(endpoint, direction, message)
      try {
        const delayed = { message, summary }
        Object.assign(summary, queue.push(delayed, frameSize(message)))
        observer.frame(summary)
      } catch (error) {
        observer.frame(summary)
        observer.failure(error instanceof Error ? error.message : 'Delayed WebSocket queue failed')
        shutdown(client, server, 1011, 'Bounded delay queue failed')
      }
    }
    client.onMessage((message) => enqueue('clientToServer', clientToServer, message))
    server.onMessage((message) => enqueue('serverToClient', serverToClient, message))
    client.onClose((code, reason) => shutdown(client, server, code, reason))
    server.onClose((code, reason) => shutdown(server, client, code, reason))
  })
  return { summary: () => current?.(), close: () => closeCurrent() }
}
