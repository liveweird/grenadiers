import { Container } from 'pixi.js'
import type { ActorDrawing } from '../src/game/arena/actorArt'
import type { AtlasRegions } from '../src/game/arena/artAtlas'
import type { ArenaArtwork } from '../src/game/arena/artwork'
import type { BombDrawing } from '../src/game/arena/entityArt'
import { graphicsArtwork } from '../src/game/arena/graphicsArtwork'
import type { ArenaScene } from '../src/game/arena/scene'
import type { Direction, Seat } from '../src/game/arena/types'
import { diagnosticSnapshot } from './snapshot'

export type ArtworkTracker = {
  actors: Map<Seat, ActorDrawing>
  bomb: BombDrawing | undefined
}

export type ActorCoverage = {
  stage: string
  appearance: Seat
  direction: Direction
  pose: number
  atlasVisible: number[]
  referenceVisible: number[]
  differentBytes: number
}

export type BombCoverage = {
  stage: string
  expectedPose: number
  atlasVisible: number[]
  referenceVisible: number[]
  differentBytes: number
}

export function trackedArtwork(base: ArenaArtwork, tracker: ArtworkTracker): ArenaArtwork {
  return {
    ...base,
    makeActor(seat) {
      const actor = base.makeActor(seat)
      tracker.actors.set(seat, actor)
      return actor
    },
    makeBomb(blast, trigger) {
      const bomb = base.makeBomb(blast, trigger)
      tracker.bomb = bomb
      return bomb
    },
  }
}

const pulsedCues = [
  'protection', 'shield', 'stopwatch', 'goldenFlame', 'shortFuse', 'shortFlame',
  'longFuse', 'reverseControls', 'diarrhea', 'cloak',
] as const satisfies ReadonlyArray<keyof ActorDrawing>

export function proceduralReferenceArtwork(
  regions: AtlasRegions,
  onBomb: (bomb: BombDrawing) => void,
): ArenaArtwork {
  return {
    ...graphicsArtwork,
    makeBomb(blast = 'standard', trigger = 'timed') {
      const bomb = graphicsArtwork.makeBomb(blast, trigger)
      const previous = bomb.shellFrames ?? [bomb.shell]
      const replacements = trigger === 'timed' ? [
        regions.createSprite(`bomb.shell.timed.${blast}`),
        regions.createSprite(`bomb.shell.timed.${blast}.1`),
        regions.createSprite(`bomb.shell.timed.${blast}.2`),
      ] as const : [regions.createSprite(`bomb.shell.${trigger}.${blast}`)] as const
      previous.forEach((shell, index) => {
        const childIndex = bomb.root.getChildIndex(shell)
        bomb.root.removeChild(shell)
        bomb.root.addChildAt(replacements[index], childIndex)
        shell.destroy({ children: true, context: true })
      })
      bomb.shell = replacements[0]
      if (replacements.length === 3) bomb.shellFrames = replacements
      else delete bomb.shellFrames
      onBomb(bomb)
      return bomb
    },
    makeActor(seat) {
      const actor = graphicsArtwork.makeActor(seat)
      for (const cue of pulsedCues) {
        const drawing = actor[cue]
        const index = actor.root.getChildIndex(drawing)
        const wrapper = new Container()
        wrapper.visible = drawing.visible
        drawing.visible = true
        drawing.alpha = 1
        actor.root.removeChild(drawing)
        wrapper.addChild(drawing)
        actor.root.addChildAt(wrapper, index)
        wrapper.cacheAsTexture({ resolution: 1, antialias: false, scaleMode: 'nearest' })
        actor[cue] = wrapper
      }
      return actor
    },
  }
}

function visibleActorFrames(actor: ActorDrawing, direction: Direction): number[] {
  return actor.poses[direction].flatMap((pose, index) => pose.visible ? [index] : [])
}

function visibleBombFrames(bomb: BombDrawing): number[] {
  return (bomb.shellFrames ?? [bomb.shell]).flatMap((shell, index) => shell.visible ? [index] : [])
}

function actorCoverage(
  stage: string,
  atlas: ArtworkTracker,
  reference: ArtworkTracker,
  direction: Direction,
  pose: number,
): Omit<ActorCoverage, 'differentBytes'>[] {
  return (['one', 'two', 'three', 'four'] as const).map((appearance) => {
    const atlasActor = atlas.actors.get(appearance)
    const referenceActor = reference.actors.get(appearance)
    if (!atlasActor || !referenceActor) throw new Error(`Missing ${appearance} actor at ${stage}`)
    const atlasVisible = visibleActorFrames(atlasActor, direction)
    const referenceVisible = visibleActorFrames(referenceActor, direction)
    if (atlasActor.direction !== direction || referenceActor.direction !== direction
      || atlasVisible.length !== 1 || atlasVisible[0] !== pose
      || referenceVisible.length !== 1 || referenceVisible[0] !== pose) {
      throw new Error(`Invalid ${appearance} actor pose at ${stage}: ${JSON.stringify({
        expected: { direction, pose },
        atlas: { direction: atlasActor.direction, visible: atlasVisible },
        reference: { direction: referenceActor.direction, visible: referenceVisible },
      })}`)
    }
    return { stage, appearance, direction, pose, atlasVisible, referenceVisible }
  })
}

export function bombCoverage(
  stage: string,
  atlas: ArtworkTracker,
  reference: ArtworkTracker,
  expectedPose: number,
): Omit<BombCoverage, 'differentBytes'> {
  if (!atlas.bomb || !reference.bomb) throw new Error(`Missing bomb at ${stage}`)
  const atlasVisible = visibleBombFrames(atlas.bomb)
  const referenceVisible = visibleBombFrames(reference.bomb)
  if (atlasVisible.length !== 1 || atlasVisible[0] !== expectedPose
    || referenceVisible.length !== 1 || referenceVisible[0] !== expectedPose) {
    throw new Error(`Invalid timed bomb pose at ${stage}`)
  }
  return { stage, expectedPose, atlasVisible, referenceVisible }
}

export function captureActorStages(
  scenes: readonly ArenaScene[],
  atlas: ArtworkTracker,
  reference: ArtworkTracker,
  capture: (stage: string) => void,
): Array<Omit<ActorCoverage, 'differentBytes'>> {
  const evidence: Array<Omit<ActorCoverage, 'differentBytes'>> = []
  scenes.forEach((scene) => scene.setReducedMotion(false))
  for (const [directionIndex, direction] of
    (['north', 'east', 'south', 'west'] as const).entries()) {
    const standing = diagnosticSnapshot(700 + directionIndex)
    standing.room.roundId = 700 + directionIndex
    standing.state.actors = (['one', 'two', 'three', 'four'] as const).map((seat, index) => ({
      id: 40 + index * 4, seat, position: { x: 1_536 + index * 3_072, y: 2_560 },
      heading: direction, alive: true,
    }))
    scenes.forEach((scene) => scene.update(standing, 0))
    const walking = structuredClone(standing)
    walking.state.physicalTick = Number(standing.state.physicalTick) + 1
    walking.state.actors.forEach((actor) => { actor.position.x += 512 })
    scenes.forEach((scene) => scene.update(walking, 1))
    for (const [cycleIndex, pose] of [0, 2, 1, 3].entries()) {
      const stage = `actor-walk-${direction}-${pose}`
      scenes.forEach((scene) => scene.render(cycleIndex * 110))
      capture(stage)
      evidence.push(...actorCoverage(stage, atlas, reference, direction, pose))
    }
    if (direction !== 'west') continue
    scenes.forEach((scene) => scene.setReducedMotion(true))
    scenes.forEach((scene) => scene.render(440))
    capture('actor-reduced-motion')
    evidence.push(...actorCoverage('actor-reduced-motion', atlas, reference, direction, 0))
    scenes.forEach((scene) => scene.setReducedMotion(false))
    const paused = structuredClone(walking)
    paused.state.physicalTick = Number(walking.state.physicalTick) + 1
    paused.state.actors.forEach((actor) => { actor.voluntaryActionsFrozen = true })
    scenes.forEach((scene) => scene.update(paused, 450))
    capture('actor-paused')
    evidence.push(...actorCoverage('actor-paused', atlas, reference, direction, 0))
    const nonRunning = structuredClone(paused)
    nonRunning.room.phase = { type: 'result' }
    scenes.forEach((scene) => scene.update(nonRunning, 460))
    capture('actor-non-running')
    evidence.push(...actorCoverage('actor-non-running', atlas, reference, direction, 0))
  }
  return evidence
}

export function withPixelEvidence<T extends { stage: string }>(
  entries: readonly T[],
  comparisons: readonly { stage: string; differentBytes: number }[],
): Array<T & { differentBytes: number }> {
  return entries.map((entry) => ({
    ...entry,
    differentBytes: comparisons.find(({ stage }) => stage === entry.stage)?.differentBytes ?? -1,
  }))
}
