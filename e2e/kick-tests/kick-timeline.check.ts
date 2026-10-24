import assert from 'node:assert/strict'
import {
  retainedKickTimeline, type KickBombIdentity, type KickSnapshot,
} from './kick-timeline.ts'

const bomb = (column: number, overrides: Partial<KickBombIdentity> = {}): KickBombIdentity => ({
  id: 7, owner: 1, range: 2, placedAt: 40, blast: 'standard',
  cell: { column, row: 1 },
  trigger: { type: 'timed', remainingTicks: 300, paused: false }, ...overrides,
})
const frame = (revision: number, value: KickBombIdentity): KickSnapshot => ({
  room: { revision }, state: { bombs: [value] },
})
const baseline = frame(10, bomb(3))

assert.deepEqual(
  retainedKickTimeline(baseline, [frame(11, bomb(2)), frame(12, bomb(1))], bomb(3))
    .map(({ cell }) => cell.column),
  [3, 2, 1],
)
assert.throws(
  () => retainedKickTimeline(baseline, [frame(11, bomb(1))], bomb(3)),
  /route mismatch/,
)
assert.throws(
  () => retainedKickTimeline(
    baseline, [frame(11, bomb(1)), frame(12, bomb(2))], bomb(3),
  ),
  /route mismatch/,
)
assert.throws(
  () => retainedKickTimeline(
    baseline, [frame(11, bomb(2, { owner: 2 })), frame(12, bomb(1))], bomb(3),
  ),
  /changed retained bomb identity/,
)

console.log('KICK_PREINPUT_TIMELINE_CAUSAL_PASS')
