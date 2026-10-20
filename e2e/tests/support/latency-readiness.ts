import { performance } from 'node:perf_hooks'
import { expect } from '@playwright/test'
import type { RoutedFrameSummary, SanitizedSnapshot } from './delayed-websocket'
import type { LatencyObserver, PublicSnapshotTiming } from './latency-observer'

export type ReadinessBaseline = {
  cutoffAt: number
  roundId: number
  ingressHighWatermark: number
  endpoints: readonly string[]
}

type ReadinessWitness = {
  snapshot: SanitizedSnapshot
  ingressAt: number
  dispatchAt: number
  pageAt: number
}

type EndpointDiagnostics = {
  reason: string
  first?: ReadinessWitness
  second?: ReadinessWitness
  lastIngress?: { snapshot: SanitizedSnapshot; at: number; dueAt?: number }
  lastDispatch?: { snapshot: SanitizedSnapshot; at: number }
  lastPage?: { snapshot: SanitizedSnapshot; at: number }
}

export type ReadinessStage = 'started' | 'join' | 'running' | 'canvas' | 'focus'
  | 'cutoff' | 'pubbaseline' | 'ready'

export type ReadinessDiagnostics = {
  startedAt: number
  stage: ReadinessStage
  baseline?: ReadinessBaseline
  evaluatedAt: number
  completedAt?: number
  ready: boolean
  reason: string
  endpoints: Record<string, EndpointDiagnostics>
}

type ReadinessObserver = Pick<LatencyObserver, 'frames' | 'forwardedFrames' | 'snapshots'>
type EndpointEvaluation = {
  diagnostics: EndpointDiagnostics
  pair?: [PublicSnapshotTiming, PublicSnapshotTiming]
  reason?: string
}

function snapshots(
  frames: readonly RoutedFrameSummary[],
  endpoint: string,
): RoutedFrameSummary[] {
  return frames.filter((frame) => frame.endpoint === endpoint
    && frame.direction === 'serverToClient' && frame.snapshot !== undefined)
}

function sameFacts(left: SanitizedSnapshot, right: SanitizedSnapshot): boolean {
  return left.revision === right.revision && left.tick === right.tick
    && left.roundId === right.roundId && left.position.x === right.position.x
    && left.position.y === right.position.y
    && left.bombIds.length === right.bombIds.length
    && left.bombIds.every((bombId, index) => bombId === right.bombIds[index])
}

function pageFacts(snapshot: PublicSnapshotTiming): SanitizedSnapshot {
  return {
    revision: snapshot.revision, tick: snapshot.tick, roundId: snapshot.roundId,
    position: snapshot.position, bombIds: snapshot.bombIds,
  }
}

function witness(
  observer: ReadinessObserver,
  baseline: ReadinessBaseline,
  endpoint: string,
  page: PublicSnapshotTiming,
): ReadinessWitness | undefined {
  const facts = pageFacts(page)
  const ingress = snapshots(observer.frames, endpoint).find((frame) => frame.at > baseline.cutoffAt
    && frame.snapshot !== undefined && sameFacts(frame.snapshot, facts))
  if (!ingress) return undefined
  const dispatch = snapshots(observer.forwardedFrames, endpoint).find((frame) => frame.at >= ingress.at
    && frame.at <= page.at && frame.snapshot !== undefined && sameFacts(frame.snapshot, facts))
  if (!dispatch) return undefined
  return { snapshot: facts, ingressAt: ingress.at, dispatchAt: dispatch.at, pageAt: page.at }
}

function pairProblem(
  pair: [PublicSnapshotTiming, PublicSnapshotTiming],
  baseline: ReadinessBaseline,
  endpoint: string,
): string | undefined {
  if (pair.some((value) => value.roundId !== baseline.roundId)) {
    return `${endpoint} publications are from the wrong round`
  }
  if (pair[0].revision >= pair[1].revision || pair[0].revision <= baseline.ingressHighWatermark) {
    return `${endpoint} publications do not advance beyond the ingress high watermark`
  }
  const stationary = pair[0].position.x === pair[1].position.x
    && pair[0].position.y === pair[1].position.y
  if (!stationary || pair.some((value) => value.bombIds.length !== 0)) {
    return `${endpoint} publications are not stationary and bomb-empty`
  }
  return undefined
}

export function captureReadinessBaseline(
  observer: ReadinessObserver,
  endpoints: readonly string[],
  cutoffAt = performance.now(),
): ReadinessBaseline {
  const heads = endpoints.map((endpoint) => snapshots(observer.frames, endpoint)
    .filter((frame) => frame.at <= cutoffAt).at(-1))
  if (heads.some((head) => head?.snapshot === undefined)) {
    throw new Error('Readiness cutoff requires a snapshot ingress head for every endpoint')
  }
  const facts = heads.map((head) => head!.snapshot!)
  if (facts.some((snapshot) => snapshot.roundId !== facts[0].roundId)) {
    throw new Error('Readiness cutoff requires one current round across endpoints')
  }
  return {
    cutoffAt, roundId: facts[0].roundId,
    ingressHighWatermark: Math.max(...facts.map((snapshot) => snapshot.revision)),
    endpoints: [...endpoints],
  }
}

function evaluateEndpoint(
  observer: ReadinessObserver,
  baseline: ReadinessBaseline,
  endpoint: string,
  evaluatedAt: number,
): EndpointEvaluation {
  const pageValues = observer.snapshots.filter((value) => value.endpoint === endpoint)
  const pair = pageValues.length >= 2
    ? [pageValues.at(-2)!, pageValues.at(-1)!] as [PublicSnapshotTiming, PublicSnapshotTiming]
    : undefined
  const ingressValues = snapshots(observer.frames, endpoint)
  const dispatchValues = snapshots(observer.forwardedFrames, endpoint)
  const first = pair && witness(observer, baseline, endpoint, pair[0])
  const second = pair && witness(observer, baseline, endpoint, pair[1])
  const diagnostics: EndpointDiagnostics = {
    reason: 'ready', first, second,
    lastIngress: ingressValues.at(-1)?.snapshot && {
      snapshot: ingressValues.at(-1)!.snapshot!, at: ingressValues.at(-1)!.at,
      dueAt: ingressValues.at(-1)!.dueAt,
    },
    lastDispatch: dispatchValues.at(-1)?.snapshot && {
      snapshot: dispatchValues.at(-1)!.snapshot!, at: dispatchValues.at(-1)!.at,
    },
    lastPage: pageValues.at(-1) && {
      snapshot: pageFacts(pageValues.at(-1)!), at: pageValues.at(-1)!.at,
    },
  }
  const rejected = (reason: string): EndpointEvaluation => ({
    diagnostics: { ...diagnostics, reason }, reason,
  })
  if (!pair) return rejected(`${endpoint} has fewer than two page publications`)
  if (!first || !second) {
    return rejected(`${endpoint} lacks fresh ingress/dispatch/page chronology`)
  }
  const problem = pairProblem(pair, baseline, endpoint)
  if (problem) return rejected(problem)
  const lastDispatch = dispatchValues.at(-1)?.snapshot
  if (!lastDispatch || !sameFacts(lastDispatch, pageFacts(pair[1]))) {
    return rejected(`${endpoint} has a newer dispatched snapshot than its latest page publication`)
  }
  const pending = ingressValues.filter((frame) => frame.snapshot!.revision > pair[1].revision)
  if (pending.some((frame) => frame.dueAt === undefined || frame.dueAt <= evaluatedAt)) {
    return rejected(`${endpoint} has an overdue ingress newer than its readiness pair`)
  }
  return { diagnostics, pair }
}

export function evaluateLatencyReadiness(
  observer: ReadinessObserver,
  baseline: ReadinessBaseline,
  evaluatedAt = performance.now(),
  startedAt = baseline.cutoffAt,
): ReadinessDiagnostics {
  const endpointDiagnostics: Record<string, EndpointDiagnostics> = {}
  const pairs = new Map<string, [PublicSnapshotTiming, PublicSnapshotTiming]>()
  const reasons: string[] = []
  for (const endpoint of baseline.endpoints) {
    const result = evaluateEndpoint(observer, baseline, endpoint, evaluatedAt)
    endpointDiagnostics[endpoint] = result.diagnostics
    if (result.pair) pairs.set(endpoint, result.pair)
    if (result.reason) reasons.push(result.reason)
  }

  const reference = pairs.get(baseline.endpoints[0])
  if (pairs.size === baseline.endpoints.length && reference) {
    for (const endpoint of baseline.endpoints.slice(1)) {
      const pair = pairs.get(endpoint)!
      if (!sameFacts(pageFacts(reference[0]), pageFacts(pair[0]))
        || !sameFacts(pageFacts(reference[1]), pageFacts(pair[1]))) {
        const mismatch = 'endpoint readiness publications do not match'
        reasons.push(mismatch)
        endpointDiagnostics[baseline.endpoints[0]].reason = mismatch
        endpointDiagnostics[endpoint].reason = mismatch
      }
    }
    const heads = baseline.endpoints.map((endpoint) => snapshots(observer.frames, endpoint).at(-1)?.snapshot)
    if (heads.some((head) => !head || head.roundId !== baseline.roundId
      || !sameFacts(head, heads[0]!) || head.revision < reference[1].revision)) {
      const mismatch = 'raw ingress heads do not agree at or beyond the readiness pair'
      reasons.push(mismatch)
      for (const endpoint of baseline.endpoints) endpointDiagnostics[endpoint].reason = mismatch
    }
  }
  const ready = pairs.size === baseline.endpoints.length && reasons.length === 0
  return {
    startedAt, stage: ready ? 'ready' : 'pubbaseline', baseline, evaluatedAt, ready,
    reason: ready ? 'ready' : reasons.join('; '), endpoints: endpointDiagnostics,
  }
}

export class LatencyReadiness {
  diagnostics: ReadinessDiagnostics
  baseline: ReadinessBaseline | undefined
  private readonly observer: ReadinessObserver
  private readonly now: () => number
  private readonly endpoints: readonly string[]
  private readonly startedAt: number

  constructor(
    observer: ReadinessObserver,
    endpointsOrBaseline: readonly string[] | ReadinessBaseline,
    now: () => number = () => performance.now(),
  ) {
    this.observer = observer
    this.now = now
    this.startedAt = now()
    if ('cutoffAt' in endpointsOrBaseline) {
      this.baseline = endpointsOrBaseline
      this.endpoints = [...endpointsOrBaseline.endpoints]
    } else {
      this.baseline = undefined
      this.endpoints = [...endpointsOrBaseline]
    }
    this.diagnostics = this.startupDiagnostics(this.baseline ? 'pubbaseline' : 'started', 'not evaluated')
  }

  setStage(stage: ReadinessStage): void {
    this.diagnostics = this.startupDiagnostics(stage, `waiting during ${stage}`)
  }

  captureBaseline(): ReadinessBaseline {
    const cutoffAt = this.now()
    this.diagnostics = this.startupDiagnostics('cutoff', 'capturing readiness cutoff', cutoffAt)
    try {
      this.baseline = captureReadinessBaseline(this.observer, this.endpoints, cutoffAt)
      this.diagnostics = this.startupDiagnostics('pubbaseline', 'waiting for publication baseline')
      return this.baseline
    } catch (error) {
      this.diagnostics = this.startupDiagnostics('cutoff', 'readiness cutoff capture failed')
      throw error
    }
  }

  fail(): void {
    const previous = this.diagnostics
    if (previous.completedAt !== undefined) return
    const reason = `profile failed during ${previous.stage}: ${previous.reason}`
    const current = this.startupDiagnostics(previous.stage, reason)
    for (const endpoint of this.endpoints) {
      current.endpoints[endpoint] = {
        ...current.endpoints[endpoint],
        reason: previous.endpoints[endpoint]?.reason === 'ready'
          ? reason : previous.endpoints[endpoint]?.reason ?? reason,
        first: previous.endpoints[endpoint]?.first,
        second: previous.endpoints[endpoint]?.second,
      }
    }
    this.diagnostics = { ...current, completedAt: previous.completedAt }
  }

  async wait(): Promise<ReadinessDiagnostics> {
    if (!this.baseline) throw new Error('Readiness baseline has not been captured')
    await expect.poll(() => {
      const evaluated = evaluateLatencyReadiness(
        this.observer, this.baseline!, this.now(), this.startedAt,
      )
      this.diagnostics = evaluated.ready
        ? { ...evaluated, completedAt: evaluated.evaluatedAt } : evaluated
      return evaluated.ready
    }, { intervals: [20] }).toBe(true)
    return this.diagnostics
  }

  private startupDiagnostics(
    stage: ReadinessStage,
    reason: string,
    evaluatedAt = this.now(),
  ): ReadinessDiagnostics {
    const endpoints: Record<string, EndpointDiagnostics> = {}
    for (const endpoint of this.endpoints) {
      const ingress = snapshots(this.observer.frames, endpoint).at(-1)
      const dispatch = snapshots(this.observer.forwardedFrames, endpoint).at(-1)
      const delivered = this.observer.snapshots.filter((value) => value.endpoint === endpoint).at(-1)
      endpoints[endpoint] = {
        reason,
        lastIngress: ingress?.snapshot && { snapshot: ingress.snapshot, at: ingress.at, dueAt: ingress.dueAt },
        lastDispatch: dispatch?.snapshot && { snapshot: dispatch.snapshot, at: dispatch.at },
        lastPage: delivered && { snapshot: pageFacts(delivered), at: delivered.at },
      }
    }
    return {
      startedAt: this.startedAt, stage, baseline: this.baseline, evaluatedAt,
      ready: false, reason, endpoints,
    }
  }
}
