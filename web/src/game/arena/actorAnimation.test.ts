import { describe, expect, it } from 'vitest'
import { selectActorPose, WALK_CYCLE, WALK_FRAME_DURATION_MS } from './actorAnimation'

describe('actor walking animation selection', () => {
  it('cycles all four players through the shared four-pose gait', () => {
    const cycles = [0, 1, 2, 3].map((actorId) =>
      [0, 1, 2, 3].map((step) =>
        selectActorPose(step * WALK_FRAME_DURATION_MS, actorId, true, true)))

    expect(WALK_CYCLE).toEqual([0, 2, 1, 3])
    expect(cycles).toEqual([
      [0, 2, 1, 3],
      [2, 1, 3, 0],
      [1, 3, 0, 2],
      [3, 0, 2, 1],
    ])
  })

  it('uses standing pose zero when movement or animation is disabled', () => {
    expect(selectActorPose(10_000, 3, false, true)).toBe(0)
    expect(selectActorPose(10_000, 3, true, false)).toBe(0)
  })
})
