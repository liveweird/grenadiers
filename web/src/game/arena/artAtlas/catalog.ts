import { Graphics, type Container } from 'pixi.js'
import { makeActor } from '../actorArt'
import { WALK_CYCLE, WALK_FRAME_DURATION_MS } from '../actorAnimation'
import { makeBomb, makeFlame, makeOilBarrel, makePickup } from '../entityArt'
import { makeBlock, makeFloorVariant, makeWallVariant } from '../terrainArt'
import { makeStormCloud } from '../weatherArt'
import { makeLair, makeMob } from '../lairArt'
import { RENDERABLE_PICKUP_KINDS, type Direction, type Seat } from '../types'
import { sourceFrame } from './layout'
import type { ArtAnimation, ArtCatalog, ArtCatalogEntry, ArtFrame, ArtRect } from './types'

type FrameOptions = {
  size?: 48 | 80
  bounds?: ArtRect
  pivot?: { x: number; y: number }
  footAnchor?: { x: number; y: number }
  opacity?: number
  status?: ArtFrame['status']
}
type DrawingFactory = () => { drawing: Container; owner?: Container }

const TILE_BOUNDS = { x: 0, y: 0, width: 48, height: 48 }
const ACTOR_BOUNDS = { x: -40, y: -44, width: 80, height: 80 }
const ACTOR_PIVOT = { x: 40, y: 44 }

function entry(id: string, slot: number, factory: DrawingFactory, options: FrameOptions = {}): ArtCatalogEntry {
  const size = options.size ?? 48
  const frame = sourceFrame(id, slot, { width: size, height: size }, options.bounds ?? TILE_BOUNDS, {
    status: options.status ?? 'baseline',
    pivot: options.pivot ?? { x: 0, y: 0 },
    ...(options.footAnchor ? { footAnchor: options.footAnchor } : {}),
    ...(options.opacity === undefined ? {} : { opacity: options.opacity }),
  })
  return {
    frame,
    createDrawing() {
      const { drawing, owner = drawing } = factory()
      drawing.visible = true
      drawing.alpha = 1
      let destroyed = false
      return {
        drawing,
        destroy() {
          if (destroyed) return
          destroyed = true
          owner.destroy({ children: true })
          if (!drawing.destroyed) drawing.destroy({ children: true })
        },
      }
    },
  }
}

function actorPart(appearance: Seat, select: (actor: ReturnType<typeof makeActor>) => Container): DrawingFactory {
  return () => {
    const actor = makeActor(appearance)
    return { drawing: select(actor), owner: actor.root }
  }
}

function bombPart(
  blast: Parameters<typeof makeBomb>[0],
  trigger: Parameters<typeof makeBomb>[1],
  select: (bomb: ReturnType<typeof makeBomb>) => Container,
): DrawingFactory {
  return () => {
    const bomb = makeBomb(blast, trigger)
    return { drawing: select(bomb), owner: bomb.root }
  }
}

function addTerrain(entries: ArtCatalogEntry[]): void {
  for (let variant = 0; variant < 8; variant += 1) {
    entries.push(entry(`terrain.floor.${variant}`, entries.length, () => ({ drawing: makeFloorVariant(variant) })))
  }
  for (let variant = 0; variant < 2; variant += 1) {
    entries.push(entry(`terrain.wall.${variant}`, entries.length, () => ({ drawing: makeWallVariant(variant) })))
  }
  entries.push(entry('terrain.block.brick', entries.length, () => ({ drawing: makeBlock() })))
}

function addActors(entries: ArtCatalogEntry[]): void {
  const directions: readonly Direction[] = ['north', 'east', 'south', 'west']
  for (const seat of ['one', 'two'] as const) {
    for (const direction of directions) {
      for (const frame of [0, 1] as const) {
        entries.push(entry(`actor.${seat}.${direction}.${frame}`, entries.length,
          actorPart(seat, (actor) => actor.poses[direction][frame]), {
            size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
          }))
      }
    }
  }
  entries.push(entry('actor.shadow', entries.length, actorPart('one', (actor) => actor.shadow), {
    size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, opacity: 0.3,
  }))
  const parts: ReadonlyArray<[string, (actor: ReturnType<typeof makeActor>) => Container]> = [
    ['protection', (actor) => actor.protection], ['shield', (actor) => actor.shield],
    ['stopwatch', (actor) => actor.stopwatch], ['frozen', (actor) => actor.frozen],
    ['goldenFlame', (actor) => actor.goldenFlame], ['shortFuse', (actor) => actor.shortFuse],
    ['shortFlame', (actor) => actor.shortFlame], ['longFuse', (actor) => actor.longFuse],
    ['reverseControls', (actor) => actor.reverseControls], ['diarrhea', (actor) => actor.diarrhea],
    ['slowBoot', (actor) => actor.slowBoot], ['piercing', (actor) => actor.piercing],
    ['power', (actor) => actor.power], ['landmine', (actor) => actor.landmine],
    ['kick', (actor) => actor.kick], ['punch', (actor) => actor.punch],
    ['bracelet', (actor) => actor.bracelet], ['wallPass', (actor) => actor.wallPass],
    ['cloak', (actor) => actor.cloak],
  ]
  for (const [id, select] of parts) {
    entries.push(entry(`actor.cue.${id}`, entries.length, actorPart('one', select), {
      size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT,
    }))
  }
  for (let marker = 0; marker < 6; marker += 1) {
    entries.push(entry(`actor.cue.slowMarker.${marker + 1}`, entries.length,
      actorPart('one', (actor) => actor.slowMarkers[marker]), {
        size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT,
      }))
  }
}

function addBombsAndEffects(entries: ArtCatalogEntry[]): void {
  const bombBounds = { x: -40, y: -40, width: 80, height: 80 }
  const options = { size: 80 as const, bounds: bombBounds, pivot: { x: 40, y: 40 } }
  entries.push(entry('bomb.shadow', entries.length,
    bombPart('standard', 'timed', (bomb) => bomb.shadow), { ...options, opacity: 0.28 }))
  for (const trigger of ['timed', 'remote'] as const) {
    for (const blast of ['standard', 'piercing', 'power'] as const) {
      entries.push(entry(`bomb.shell.${trigger}.${blast}`, entries.length,
        bombPart(blast, trigger, (bomb) => bomb.shell), options))
    }
  }
  entries.push(entry('bomb.shell.contact.standard', entries.length,
    bombPart('standard', 'contact', (bomb) => bomb.shell), options))
  entries.push(entry('bomb.spark', entries.length,
    bombPart('standard', 'timed', (bomb) => bomb.spark), options))
  entries.push(entry('bomb.pause', entries.length,
    bombPart('standard', 'timed', (bomb) => bomb.pause), options))
  for (const frame of [0, 1] as const) {
    entries.push(entry(`effect.flame.${frame}`, entries.length, () => {
      const flame = makeFlame()
      return { drawing: flame.frames[frame], owner: flame.root }
    }))
  }
  entries.push(entry('oil.unlit', entries.length, () => {
    const oil = makeOilBarrel(); return { drawing: oil.unlit, owner: oil.root }
  }))
  entries.push(entry('oil.charred', entries.length, () => {
    const oil = makeOilBarrel(); return { drawing: oil.charred, owner: oil.root }
  }))
}

function addPickupsAndReservations(entries: ArtCatalogEntry[]): void {
  for (const kind of RENDERABLE_PICKUP_KINDS) {
    entries.push(entry(`pickup.${kind}`, entries.length, () => ({ drawing: makePickup(kind) })))
  }
  for (const id of ['actor.one.death', 'actor.two.death', 'terrain.block.destruction', 'pickup.collection']) {
    entries.push(entry(`reserved.${id}`, entries.length, () => ({ drawing: new Graphics() }), {
      status: 'reserved',
    }))
  }
}

function addAdditionalActorAppearances(entries: ArtCatalogEntry[]): void {
  const directions: readonly Direction[] = ['north', 'east', 'south', 'west']
  for (const appearance of ['three', 'four'] as const) {
    for (const direction of directions) {
      for (const frame of [0, 1] as const) {
        entries.push(entry(`actor.${appearance}.${direction}.${frame}`, entries.length,
          actorPart(appearance, (actor) => actor.poses[direction][frame]), {
            size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
          }))
      }
    }
  }
}

function addWalkingFrames(entries: ArtCatalogEntry[]): void {
  const directions: readonly Direction[] = ['north', 'east', 'south', 'west']
  for (const appearance of ['one', 'two', 'three', 'four'] as const) {
    for (const direction of directions) {
      for (const frame of [2, 3] as const) {
        entries.push(entry(`actor.${appearance}.${direction}.${frame}`, entries.length,
          actorPart(appearance, (actor) => actor.poses[direction][frame]), {
            size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
          }))
      }
    }
  }
}

function addTimedBombPoses(entries: ArtCatalogEntry[]): void {
  const options = {
    size: 80 as const, bounds: { x: -40, y: -40, width: 80, height: 80 },
    pivot: { x: 40, y: 40 },
  }
  for (const blast of ['standard', 'piercing', 'power'] as const) {
    for (const frame of [1, 2] as const) {
      entries.push(entry(`bomb.shell.timed.${blast}.${frame}`, entries.length,
        bombPart(blast, 'timed', (bomb) => bomb.shellFrames![frame]), options))
    }
  }
}

function animations(): Readonly<Record<string, ArtAnimation>> {
  const result: Record<string, ArtAnimation> = {
    flame: { frameIds: ['effect.flame.0', 'effect.flame.1'], frameDurationMs: 80, reducedMotionFrame: 'effect.flame.0' },
    spark: { frameIds: ['bomb.spark'], frameDurationMs: 90, reducedMotionFrame: 'bomb.spark' },
  }
  for (const seat of ['one', 'two', 'three', 'four'] as const) {
    for (const direction of ['north', 'east', 'south', 'west'] as const) {
      const prefix = `actor.${seat}.${direction}`
      result[`${prefix}.walk`] = {
        frameIds: WALK_CYCLE.map((frame) => `${prefix}.${frame}`),
        frameDurationMs: WALK_FRAME_DURATION_MS, reducedMotionFrame: `${prefix}.0`,
      }
    }
  }
  for (const blast of ['standard', 'piercing', 'power'] as const) {
    result[`bomb.timed.${blast}.pulse`] = {
      frameIds: [
        `bomb.shell.timed.${blast}`, `bomb.shell.timed.${blast}.1`, `bomb.shell.timed.${blast}.2`,
      ],
      frameDurationMs: 110,
      reducedMotionFrame: `bomb.shell.timed.${blast}`,
    }
  }
  return result
}

export function createArtCatalog(): ArtCatalog {
  const entries: ArtCatalogEntry[] = []
  addTerrain(entries)
  addActors(entries)
  addBombsAndEffects(entries)
  addPickupsAndReservations(entries)
  addAdditionalActorAppearances(entries)
  addWalkingFrames(entries)
  addTimedBombPoses(entries)
  entries.push(entry('environment.stormCloud', entries.length,
    () => ({ drawing: makeStormCloud() }), {
      size: 80, bounds: { x: -40, y: -40, width: 80, height: 80 }, pivot: { x: 40, y: 40 },
    }))
  entries.push(entry('terrain.lair', entries.length, () => ({ drawing: makeLair() })))
  for (const direction of ['north', 'east', 'south', 'west'] as const) {
    entries.push(entry(`mob.${direction}`, entries.length,
      () => ({ drawing: makeMob(direction) }), {
        size: 80, bounds: ACTOR_BOUNDS, pivot: ACTOR_PIVOT, footAnchor: { x: 40, y: 65 },
      }))
  }
  return {
    entries,
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
