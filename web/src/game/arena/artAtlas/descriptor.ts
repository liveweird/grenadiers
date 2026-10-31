import { sourceFrame } from './layout'
import type { ArtAnimation, ArtFrame, ArtManifest, ArtRect } from './types'

type FrameOptions = {
  size?: 48 | 80
  bounds?: ArtRect
  pivot?: { x: number; y: number }
  footAnchor?: { x: number; y: number }
  opacity?: number
  status?: ArtFrame['status']
}

export const MATERIAL_FRAME_IDS = [
  'material.floor.top.0', 'material.floor.top.1',
  'material.wall.top', 'material.wall.side',
  'material.block.top', 'material.block.side',
  'material.lair.top', 'material.lair.side',
  'material.rim.top', 'material.rim.side',
] as const

const TILE_BOUNDS = { x: 0, y: 0, width: 48, height: 48 }
const ACTOR_BOUNDS = { x: -40, y: -44, width: 80, height: 80 }
const ACTOR_PIVOT = { x: 40, y: 44 }
const DIRECTIONS = ['north', 'east', 'south', 'west'] as const
const APPEARANCES = ['one', 'two', 'three', 'four'] as const
const PICKUPS = [
  'bombCapacity', 'blastRange', 'movementSpeed', 'kick', 'punch', 'bracelet', 'wallPass', 'heart',
  'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
] as const

function addFrame(frames: ArtFrame[], id: string, options: FrameOptions = {}): void {
  const size = options.size ?? 48
  frames.push(sourceFrame(
    id, frames.length, { width: size, height: size }, options.bounds ?? TILE_BOUNDS,
    {
      status: options.status ?? 'baseline',
      pivot: options.pivot ?? { x: 0, y: 0 },
      ...(options.footAnchor ? { footAnchor: options.footAnchor } : {}),
      ...(options.opacity === undefined ? {} : { opacity: options.opacity }),
    },
  ))
}

function addOriginalFrames(frames: ArtFrame[]): void {
  for (let variant = 0; variant < 8; variant += 1) addFrame(frames, `terrain.floor.${variant}`)
  for (let variant = 0; variant < 2; variant += 1) addFrame(frames, `terrain.wall.${variant}`)
  addFrame(frames, 'terrain.block.brick')
  for (const appearance of ['one', 'two'] as const) {
    for (const direction of DIRECTIONS) {
      for (const pose of [0, 1] as const) addActorFrame(frames, appearance, direction, pose)
    }
  }
  addFrame(frames, 'actor.shadow', {
    size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, opacity: 0.3,
  })
  for (const cue of [
    'protection', 'shield', 'stopwatch', 'frozen', 'goldenFlame', 'shortFuse', 'shortFlame',
    'longFuse', 'reverseControls', 'diarrhea', 'slowBoot', 'piercing', 'power', 'landmine',
    'kick', 'punch', 'bracelet', 'wallPass', 'cloak',
  ]) addFrame(frames, `actor.cue.${cue}`, { size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT })
  for (let marker = 1; marker <= 6; marker += 1) {
    addFrame(frames, `actor.cue.slowMarker.${marker}`, {
      size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT,
    })
  }
  addBombAndEffectFrames(frames)
  for (const pickup of PICKUPS) addFrame(frames, `pickup.${pickup}`)
  for (const id of [
    'actor.one.death', 'actor.two.death', 'terrain.block.destruction', 'pickup.collection',
  ]) addFrame(frames, `reserved.${id}`, { status: 'reserved' })
}

function addActorFrame(
  frames: ArtFrame[], appearance: string, direction: string, pose: number,
): void {
  addFrame(frames, `actor.${appearance}.${direction}.${pose}`, {
    size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
  })
}

function addBombAndEffectFrames(frames: ArtFrame[]): void {
  const bomb = { size: 80 as const, bounds: { x: -40, y: -40, width: 80, height: 80 }, pivot: { x: 40, y: 40 } }
  addFrame(frames, 'bomb.shadow', { ...bomb, opacity: 0.28 })
  for (const trigger of ['timed', 'remote'] as const) {
    for (const blast of ['standard', 'piercing', 'power'] as const) {
      addFrame(frames, `bomb.shell.${trigger}.${blast}`, bomb)
    }
  }
  addFrame(frames, 'bomb.shell.contact.standard', bomb)
  addFrame(frames, 'bomb.spark', bomb)
  addFrame(frames, 'bomb.pause', bomb)
  addFrame(frames, 'effect.flame.0')
  addFrame(frames, 'effect.flame.1')
  addFrame(frames, 'oil.unlit')
  addFrame(frames, 'oil.charred')
}

function addExtensions(frames: ArtFrame[]): void {
  for (const appearance of ['three', 'four'] as const) {
    for (const direction of DIRECTIONS) {
      for (const pose of [0, 1] as const) addActorFrame(frames, appearance, direction, pose)
    }
  }
  for (const appearance of APPEARANCES) {
    for (const direction of DIRECTIONS) {
      for (const pose of [2, 3] as const) addActorFrame(frames, appearance, direction, pose)
    }
  }
  const bomb = { size: 80 as const, bounds: { x: -40, y: -40, width: 80, height: 80 }, pivot: { x: 40, y: 40 } }
  for (const blast of ['standard', 'piercing', 'power'] as const) {
    for (const pose of [1, 2] as const) addFrame(frames, `bomb.shell.timed.${blast}.${pose}`, bomb)
  }
  addFrame(frames, 'environment.stormCloud', {
    size: 80, bounds: { x: -40, y: -40, width: 80, height: 80 }, pivot: { x: 40, y: 40 },
  })
  addFrame(frames, 'terrain.lair')
  for (const direction of DIRECTIONS) addFrame(frames, `mob.${direction}`, {
    size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
  })
  for (const id of MATERIAL_FRAME_IDS) addFrame(frames, id)
}

function addDeathFrames(frames: ArtFrame[]): void {
  for (const appearance of APPEARANCES) {
    for (const pose of [0, 1, 2, 3] as const) {
      addFrame(frames, `actor.${appearance}.death.${pose}`, {
        size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
      })
    }
  }
  for (const pose of [0, 1, 2, 3] as const) {
    addFrame(frames, `mob.death.${pose}`, {
      size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
    })
  }
}

function animations(): Readonly<Record<string, ArtAnimation>> {
  const result: Record<string, ArtAnimation> = {
    flame: { frameIds: ['effect.flame.0', 'effect.flame.1'], frameDurationMs: 80, reducedMotionFrame: 'effect.flame.0' },
    spark: { frameIds: ['bomb.spark'], frameDurationMs: 90, reducedMotionFrame: 'bomb.spark' },
  }
  for (const appearance of APPEARANCES) {
    for (const direction of DIRECTIONS) {
      const prefix = `actor.${appearance}.${direction}`
      result[`${prefix}.walk`] = {
        frameIds: [0, 2, 1, 3].map((pose) => `${prefix}.${pose}`),
        frameDurationMs: 110, reducedMotionFrame: `${prefix}.0`,
      }
    }
    const prefix = `actor.${appearance}.death`
    result[prefix] = {
      frameIds: [0, 1, 2, 3].map((pose) => `${prefix}.${pose}`),
      frameDurationMs: 150, reducedMotionFrame: `${prefix}.3`,
    }
  }
  for (const blast of ['standard', 'piercing', 'power'] as const) result[`bomb.timed.${blast}.pulse`] = {
    frameIds: [`bomb.shell.timed.${blast}`, `bomb.shell.timed.${blast}.1`, `bomb.shell.timed.${blast}.2`],
    frameDurationMs: 110, reducedMotionFrame: `bomb.shell.timed.${blast}`,
  }
  result['mob.death'] = {
    frameIds: [0, 1, 2, 3].map((pose) => `mob.death.${pose}`),
    frameDurationMs: 150, reducedMotionFrame: 'mob.death.3',
  }
  return result
}

export type ArtDescriptor = Pick<ArtManifest, 'frames' | 'animations' | 'aliases' | 'compositions'>

export function createArtDescriptor(): ArtDescriptor {
  const frames: ArtFrame[] = []
  addOriginalFrames(frames)
  addExtensions(frames)
  addDeathFrames(frames)
  return {
    frames,
    animations: animations(),
    aliases: {
      'bomb.shell.contact.piercing': 'bomb.shell.contact.standard',
      'bomb.shell.contact.power': 'bomb.shell.contact.standard',
    },
    compositions: {
      'oil.burning.0': ['oil.charred', 'effect.flame.0'],
      'oil.burning.1': ['oil.charred', 'effect.flame.1'],
    },
  }
}
