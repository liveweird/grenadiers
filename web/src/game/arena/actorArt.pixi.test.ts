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

  it('keeps simultaneous effect cues stable while selecting snapshot frame one', () => {
    const actor = makeActor('two')
    const cues = [
      actor.protection, actor.shield, actor.goldenFlame, actor.shortFuse, actor.shortFlame,
      actor.longFuse, actor.reverseControls,
      actor.diarrhea,
    ]
    for (const cue of cues) cue.visible = true
    const changes: boolean[] = []
    for (const cue of cues) cue.on('visibleChanged', (visible) => changes.push(Boolean(visible)))

    setActorPose(actor, 'west', 1)
    setActorPose(actor, 'west', 1)

    expect(changes).toEqual([])
    expect(cues.every(({ visible }) => visible)).toBe(true)
    expect(Object.values(actor.poses).flat().filter(({ visible }) => visible)).toEqual([actor.poses.west[1]])
  })
})
