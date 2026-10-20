import { expect, test } from '@playwright/test'
import type { RoutedFrameSummary, SanitizedSnapshot } from '../tests/support/delayed-websocket'
import {
  captureReadinessBaseline, evaluateLatencyReadiness, LatencyReadiness, type ReadinessBaseline,
} from '../tests/support/latency-readiness'
import {
  LatencyObserver, sample, type PublicSnapshotTiming, type SanitizedArena,
} from '../tests/support/latency-observer'

const endpoints = ['actor', 'peer'] as const
const baseline: ReadinessBaseline = {
  cutoffAt: 100, roundId: 1, ingressHighWatermark: 10, endpoints,
}

function facts(revision: number, roundId = 1, x = 1_536): SanitizedSnapshot {
  return { revision, tick: revision * 2, roundId, position: { x, y: 1_536 }, bombIds: [] }
}

function frame(
  endpoint: string,
  revision: number,
  at: number,
  options: { dueAt?: number; roundId?: number; x?: number } = {},
): RoutedFrameSummary {
  const { dueAt, roundId = 1, x = 1_536 } = options
  return {
    endpoint, direction: 'serverToClient', at, bytes: 100, type: 'snapshot',
    snapshot: facts(revision, roundId, x), dueAt,
  }
}

function page(
  endpoint: string,
  revision: number,
  at: number,
  roundId = 1,
  x = 1_536,
): PublicSnapshotTiming {
  return { endpoint, at, ...facts(revision, roundId, x) }
}

function caughtUp(): LatencyObserver {
  const observer = new LatencyObserver()
  for (const endpoint of endpoints) {
    observer.frames.push(frame(endpoint, 11, 110), frame(endpoint, 12, 120))
    observer.forwardedFrames.push(frame(endpoint, 11, 130), frame(endpoint, 12, 140))
    observer.snapshots.push(page(endpoint, 11, 150), page(endpoint, 12, 160))
  }
  return observer
}

function mutateFacts(
  observer: LatencyObserver,
  endpoint: string,
  change: (value: SanitizedSnapshot) => void,
): void {
  for (const value of [...observer.frames, ...observer.forwardedFrames]) {
    if (value.endpoint === endpoint && value.snapshot) change(value.snapshot)
  }
  for (const value of observer.snapshots) {
    if (value.endpoint === endpoint) change(value)
  }
}

test('readiness rejects publications ingressed before its cutoff', () => {
  const observer = caughtUp()
  for (const value of observer.frames) value.at -= 20
  expect(evaluateLatencyReadiness(observer, baseline, 170)).toMatchObject({
    ready: false,
    endpoints: {
      actor: { reason: 'actor lacks fresh ingress/dispatch/page chronology' },
      peer: { reason: 'peer lacks fresh ingress/dispatch/page chronology' },
    },
  })
})

test('readiness rejects only one fresh revision', () => {
  const observer = caughtUp()
  observer.snapshots.splice(0, 1)
  expect(evaluateLatencyReadiness(observer, baseline, 170)).toMatchObject({
    ready: false, reason: 'actor has fewer than two page publications',
  })
})

test('readiness rejects missing ingress, dispatch, wrong endpoint, and wrong round evidence', () => {
  const missingIngress = caughtUp()
  missingIngress.frames.splice(0, 1)
  expect(evaluateLatencyReadiness(missingIngress, baseline, 170).ready).toBe(false)

  const missingDispatch = caughtUp()
  missingDispatch.forwardedFrames.splice(0, 1)
  expect(evaluateLatencyReadiness(missingDispatch, baseline, 170).ready).toBe(false)

  const wrongEndpoint = caughtUp()
  wrongEndpoint.forwardedFrames[0].endpoint = 'peer'
  expect(evaluateLatencyReadiness(wrongEndpoint, baseline, 170).ready).toBe(false)

  const wrongRound = caughtUp()
  wrongRound.snapshots[0] = page('actor', 11, 150, 2)
  expect(evaluateLatencyReadiness(wrongRound, baseline, 170).ready).toBe(false)
})

test('readiness rejects an old matching pair when raw ingress or dispatch is ahead', () => {
  const rawAhead = caughtUp()
  for (const endpoint of endpoints) rawAhead.frames.push(frame(endpoint, 13, 165))
  expect(evaluateLatencyReadiness(rawAhead, baseline, 170)).toMatchObject({
    ready: false,
    endpoints: {
      actor: { reason: 'actor has an overdue ingress newer than its readiness pair' },
      peer: { reason: 'peer has an overdue ingress newer than its readiness pair' },
    },
  })

  const dispatchAhead = caughtUp()
  for (const endpoint of endpoints) {
    dispatchAhead.frames.push(frame(endpoint, 13, 165, { dueAt: 300 }))
    dispatchAhead.forwardedFrames.push(frame(endpoint, 13, 166))
  }
  expect(evaluateLatencyReadiness(dispatchAhead, baseline, 170).ready).toBe(false)

  const nextRoundAhead = caughtUp()
  for (const endpoint of endpoints) {
    nextRoundAhead.frames.push(frame(endpoint, 13, 165, { dueAt: 300, roundId: 2 }))
  }
  expect(evaluateLatencyReadiness(nextRoundAhead, baseline, 170)).toMatchObject({
    ready: false, reason: 'raw ingress heads do not agree at or beyond the readiness pair',
  })
})

test('readiness accepts a caught-up pair with ordinary scheduled future frames', () => {
  const observer = caughtUp()
  for (const endpoint of endpoints) observer.frames.push(frame(endpoint, 13, 165, { dueAt: 300 }))
  const result = evaluateLatencyReadiness(observer, baseline, 170)
  expect(result).toMatchObject({
    ready: true, reason: 'ready',
    endpoints: {
      actor: {
        reason: 'ready',
        first: { snapshot: facts(11), ingressAt: 110, dispatchAt: 130, pageAt: 150 },
        second: { snapshot: facts(12), ingressAt: 120, dispatchAt: 140, pageAt: 160 },
        lastIngress: { snapshot: facts(13), at: 165, dueAt: 300 },
        lastDispatch: { snapshot: facts(12), at: 140 },
        lastPage: { snapshot: facts(12), at: 160 },
      },
      peer: { reason: 'ready', first: { snapshot: facts(11) }, second: { snapshot: facts(12) } },
    },
  })
})

test('startup readiness diagnostics survive a cutoff capture failure', () => {
  const observer = new LatencyObserver()
  observer.frames.push(frame('actor', 8, 90))
  const readiness = new LatencyReadiness(observer, endpoints, () => 100)
  readiness.setStage('join')
  readiness.setStage('running')
  readiness.setStage('canvas')
  readiness.setStage('focus')
  expect(() => readiness.captureBaseline()).toThrow(
    'Readiness cutoff requires a snapshot ingress head for every endpoint',
  )
  expect(readiness.diagnostics).toMatchObject({
    startedAt: 100, stage: 'cutoff', ready: false,
    reason: 'readiness cutoff capture failed',
    endpoints: {
      actor: { reason: 'readiness cutoff capture failed', lastIngress: { snapshot: facts(8), at: 90 } },
      peer: { reason: 'readiness cutoff capture failed' },
    },
  })
})

test('readiness rejects moving or bomb-filled publication pairs', () => {
  const moving = caughtUp()
  for (const value of [...moving.frames, ...moving.forwardedFrames]) {
    if (value.endpoint === 'actor' && value.snapshot?.revision === 12) value.snapshot.position.x = 1_600
  }
  moving.snapshots.find((value) => value.endpoint === 'actor' && value.revision === 12)!.position.x = 1_600
  expect(evaluateLatencyReadiness(moving, baseline, 170).endpoints.actor.reason)
    .toBe('actor publications are not stationary and bomb-empty')

  const bombFilled = caughtUp()
  mutateFacts(bombFilled, 'actor', (value) => value.bombIds.push(7))
  expect(evaluateLatencyReadiness(bombFilled, baseline, 170).endpoints.actor.reason)
    .toBe('actor publications are not stationary and bomb-empty')
})

test('readiness rejects cross-endpoint publication fact mismatches', () => {
  const observer = caughtUp()
  mutateFacts(observer, 'peer', (value) => { value.position.x = 1_600 })
  for (const endpoint of endpoints) observer.frames.push(frame(endpoint, 13, 165, { dueAt: 300 }))
  const result = evaluateLatencyReadiness(observer, baseline, 170)
  expect(result).toMatchObject({
    ready: false, reason: 'endpoint readiness publications do not match',
    endpoints: {
      actor: { reason: 'endpoint readiness publications do not match' },
      peer: { reason: 'endpoint readiness publications do not match' },
    },
  })
})

test('readiness never passes a stalled endpoint as time advances', () => {
  const observer = caughtUp()
  observer.snapshots.pop()
  expect(evaluateLatencyReadiness(observer, baseline, 170).ready).toBe(false)
  expect(evaluateLatencyReadiness(observer, baseline, 10_000).ready).toBe(false)
})

test('a synthetic 300ms first movement after readiness remains the first witness and an exceedance', async () => {
  const observer = caughtUp()
  const gate = new LatencyReadiness(observer, baseline, () => 170)
  const readiness = await gate.wait()
  expect(readiness).toMatchObject({ ready: true, completedAt: 170 })
  const accepted = structuredClone(readiness)
  gate.fail()
  expect(gate.diagnostics).toEqual(accepted)
  const arena: SanitizedArena = {
    width: 15, height: 13,
    terrain: Array.from({ length: 15 * 13 }, () => 'floor'), blocks: [],
  }
  observer.arenas.set('peer', arena)
  observer.frames.push(frame('peer', 13, 1_110, { x: 1_600 }),
    frame('peer', 14, 1_120, { x: 1_664 }))
  observer.snapshots.push(page('peer', 13, 1_300, 1, 1_600), page('peer', 14, 1_310, 1, 1_664))
  const [first] = await observer.twoMoved('peer', 1_100, page('peer', 12, 160), 'east')
  const measured = sample({
    kind: 'action', direction: 'east', keyStart: 1_000, keyFinished: 1_001,
    outgoingAt: 1_100, observedAt: first.at,
  })
  expect(measured.keyStart).toBeGreaterThan(readiness.completedAt!)
  expect(first.revision).toBe(13)
  expect(measured.latencyMs).toBe(300)
  expect(measured.latencyMs).toBeGreaterThan(250)
})

test('readiness baseline fixes cutoff, round, and maximum ingress revision', () => {
  const observer = new LatencyObserver()
  observer.frames.push(frame('actor', 8, 80), frame('peer', 10, 90))
  expect(captureReadinessBaseline(observer, endpoints, 100)).toEqual(baseline)
})
