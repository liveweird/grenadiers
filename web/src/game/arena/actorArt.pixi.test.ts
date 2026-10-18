import { describe, expect, it } from 'vitest'
import { makeActor, setActorPose } from './actorArt'

describe('real Pixi actor pose visibility', () => {
  it('emits changes only when the selected direction or frame changes', () => {
    const actor = makeActor('one')
    const changes: Array<{ pose: string; visible: boolean }> = []
    for (const [direction, poses] of Object.entries(actor.poses)) {
      poses.forEach((pose, frame) => {
        pose.on('visibleChanged', (visible) => changes.push({
          pose: `${direction}:${frame}`, visible: Boolean(visible),
        }))
      })
    }

    setActorPose(actor, 'south', 0)
    expect(changes).toEqual([])

    setActorPose(actor, 'north', 1)
    expect(changes).toEqual([
      { pose: 'north:1', visible: true },
      { pose: 'south:0', visible: false },
    ])
    expect(Object.values(actor.poses).flat().filter(({ visible }) => visible)).toHaveLength(1)
  })
})
