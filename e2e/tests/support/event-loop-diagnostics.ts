import { monitorEventLoopDelay, performance } from 'node:perf_hooks'

const NANOS_PER_MILLISECOND = 1_000_000

export type EventLoopDelaySummary = {
  timeOrigin: number
  resolutionMs: number
  count: number
  minMs: number | null
  maxMs: number | null
  meanMs: number | null
  p50Ms: number | null
  p95Ms: number | null
  p99Ms: number | null
}

export class EventLoopDelayMonitor {
  private readonly histogram = monitorEventLoopDelay({ resolution: 10 })

  constructor() { this.histogram.enable() }

  summary(): EventLoopDelaySummary {
    const count = this.histogram.count
    const milliseconds = (value: number): number | null => count === 0
      ? null
      : value / NANOS_PER_MILLISECOND
    return {
      timeOrigin: performance.timeOrigin,
      resolutionMs: 10,
      count,
      minMs: milliseconds(this.histogram.min),
      maxMs: milliseconds(this.histogram.max),
      meanMs: milliseconds(this.histogram.mean),
      p50Ms: milliseconds(this.histogram.percentile(50)),
      p95Ms: milliseconds(this.histogram.percentile(95)),
      p99Ms: milliseconds(this.histogram.percentile(99)),
    }
  }

  stop(): void { this.histogram.disable() }
}
