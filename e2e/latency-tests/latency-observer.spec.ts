import { expect, test } from '@playwright/test'
import type { RoutedFrameSummary, SanitizedSnapshot } from '../tests/support/delayed-websocket'
import {
  LatencyObserver, type PublicSnapshotTiming, type SanitizedArena,
} from '../tests/support/latency-observer'

const arena = (): SanitizedArena => ({
  width: 15,
  height: 13,
  terrain: Array.from({ length: 15 * 13 }, (_value, index) => index % 15 === 0 ? 'wall' : 'floor'),
  blocks: [],
})

const facts = (revision: number, x: number): SanitizedSnapshot => ({
  revision, tick: revision, roundId: 1, position: { x, y: 1_536 }, bombIds: [],
})

const ingress = (revision: number, x: number, at: number): RoutedFrameSummary => ({
  endpoint: 'peer', direction: 'serverToClient', at, bytes: 100, type: 'snapshot',
  snapshot: facts(revision, x),
})

const delivered = (revision: number, x: number, at: number): PublicSnapshotTiming => ({
  endpoint: 'peer', at, ...facts(revision, x),
})

test('page delivery evidence requires a prior delayed downlink dispatch', () => {
  const observer = new LatencyObserver()
  const beforeDispatch = delivered(2, 1_600, 99)
  observer.forwarded({ ...ingress(2, 1_600, 100), at: 100 })
  expect(observer.pageDeliveryHasPriorDispatch('peer', beforeDispatch)).toBe(false)
  expect(observer.pageDeliveryHasPriorDispatch('peer', delivered(2, 1_600, 101))).toBe(true)
  expect(observer.pageDeliveryHasPriorDispatch('actor', delivered(2, 1_600, 101))).toBe(false)
})

test('release evidence excludes stable snapshots queued before null dispatch', async () => {
  const observer = new LatencyObserver()
  observer.arenas.set('peer', arena())
  observer.frames.push(ingress(2, 1_664, 90), ingress(3, 1_664, 91))
  observer.snapshots.push(delivered(2, 1_664, 101), delivered(3, 1_664, 102))

  const result = observer.firstStablePair('peer', 100, 1, 'east')
  setTimeout(() => {
    observer.frames.push(ingress(4, 1_728, 101), ingress(5, 1_728, 102))
    observer.snapshots.push(delivered(4, 1_728, 103), delivered(5, 1_728, 104))
  }, 1)

  await expect(result).resolves.toEqual({
    observed: delivered(4, 1_728, 103), confirmed: delivered(5, 1_728, 104),
  })
})

test('release evidence rejects a stable pair at a wall boundary', async () => {
  const observer = new LatencyObserver()
  observer.arenas.set('peer', arena())
  observer.frames.push(ingress(2, 1_344, 101), ingress(3, 1_344, 102))
  observer.snapshots.push(delivered(2, 1_344, 103), delivered(3, 1_344, 104))

  const result = observer.firstStablePair('peer', 100, 1, 'west')
  setTimeout(() => {
    observer.frames.push(ingress(4, 1_536, 105), ingress(5, 1_536, 106))
    observer.snapshots.push(delivered(4, 1_536, 107), delivered(5, 1_536, 108))
  }, 1)

  await expect(result).resolves.toEqual({
    observed: delivered(4, 1_536, 107), confirmed: delivered(5, 1_536, 108),
  })
})

test('trajectory chooses the greatest public collision-free clearance', () => {
  const observer = new LatencyObserver()
  const corridor = arena()
  corridor.blocks.push({ column: 7, row: 1 })
  observer.arenas.set('peer', corridor)
  observer.snapshots.push(delivered(10, 1_792, 100))
  expect(observer.safestHorizontalDirection('peer')).toEqual({
    direction: 'east', position: { x: 1_792, y: 1_536 },
    eastClearance: 5_056, westClearance: 448,
  })

  observer.snapshots.push(delivered(11, 6_000, 101))
  expect(observer.safestHorizontalDirection('peer')).toEqual({
    direction: 'west', position: { x: 6_000, y: 1_536 },
    eastClearance: 848, westClearance: 4_656,
  })
})
