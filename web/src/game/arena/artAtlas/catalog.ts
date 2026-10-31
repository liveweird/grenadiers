import { Graphics, type Container } from 'pixi.js'
import { makeActor } from '../actorArt'
import { makeBomb, makeFlame, makeOilBarrel, makePickup } from '../entityArt'
import { makeBlock, makeFloorVariant, makeWallVariant } from '../terrainArt'
import { makeStormCloud } from '../weatherArt'
import { makeLair, makeMob } from '../lairArt'
import { RENDERABLE_PICKUP_KINDS, type Direction, type Seat } from '../types'
import { createArtDescriptor, MATERIAL_FRAME_IDS } from './descriptor'
import { makeMaterialFrame } from './materialArtwork'
import type { ArtCatalog, ArtCatalogEntry } from './types'

type DrawingFactory = () => { drawing: Container; owner?: Container }

const DESCRIPTOR = createArtDescriptor()

function entry(id: string, slot: number, factory: DrawingFactory): ArtCatalogEntry {
  const frame = DESCRIPTOR.frames[slot]
  if (!frame || frame.id !== id) throw new Error(`Drawing catalogue slot ${slot} must provide ${frame?.id ?? 'no frame'}, got ${id}`)
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
          actorPart(seat, (actor) => actor.poses[direction][frame])))
      }
    }
  }
  entries.push(entry('actor.shadow', entries.length, actorPart('one', (actor) => actor.shadow)))
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
    entries.push(entry(`actor.cue.${id}`, entries.length, actorPart('one', select)))
  }
  for (let marker = 0; marker < 6; marker += 1) {
    entries.push(entry(`actor.cue.slowMarker.${marker + 1}`, entries.length,
      actorPart('one', (actor) => actor.slowMarkers[marker])))
  }
}

function addBombsAndEffects(entries: ArtCatalogEntry[]): void {
  entries.push(entry('bomb.shadow', entries.length,
    bombPart('standard', 'timed', (bomb) => bomb.shadow)))
  for (const trigger of ['timed', 'remote'] as const) {
    for (const blast of ['standard', 'piercing', 'power'] as const) {
      entries.push(entry(`bomb.shell.${trigger}.${blast}`, entries.length,
        bombPart(blast, trigger, (bomb) => bomb.shell)))
    }
  }
  entries.push(entry('bomb.shell.contact.standard', entries.length,
    bombPart('standard', 'contact', (bomb) => bomb.shell)))
  entries.push(entry('bomb.spark', entries.length,
    bombPart('standard', 'timed', (bomb) => bomb.spark)))
  entries.push(entry('bomb.pause', entries.length,
    bombPart('standard', 'timed', (bomb) => bomb.pause)))
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
    entries.push(entry(`reserved.${id}`, entries.length, () => ({ drawing: new Graphics() })))
  }
}

function addAdditionalActorAppearances(entries: ArtCatalogEntry[]): void {
  const directions: readonly Direction[] = ['north', 'east', 'south', 'west']
  for (const appearance of ['three', 'four'] as const) {
    for (const direction of directions) {
      for (const frame of [0, 1] as const) {
        entries.push(entry(`actor.${appearance}.${direction}.${frame}`, entries.length,
          actorPart(appearance, (actor) => actor.poses[direction][frame])))
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
          actorPart(appearance, (actor) => actor.poses[direction][frame])))
      }
    }
  }
}

function addTimedBombPoses(entries: ArtCatalogEntry[]): void {
  for (const blast of ['standard', 'piercing', 'power'] as const) {
    for (const frame of [1, 2] as const) {
      entries.push(entry(`bomb.shell.timed.${blast}.${frame}`, entries.length,
        bombPart(blast, 'timed', (bomb) => bomb.shellFrames![frame])))
    }
  }
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
    () => ({ drawing: makeStormCloud() })))
  entries.push(entry('terrain.lair', entries.length, () => ({ drawing: makeLair() })))
  for (const direction of ['north', 'east', 'south', 'west'] as const) {
    entries.push(entry(`mob.${direction}`, entries.length,
      () => ({ drawing: makeMob(direction) })))
  }
  for (const id of MATERIAL_FRAME_IDS) {
    entries.push(entry(id, entries.length, () => ({ drawing: makeMaterialFrame(id) })))
  }
  if (entries.length !== DESCRIPTOR.frames.length) {
    throw new Error(`Drawing catalogue supplies ${entries.length} of ${DESCRIPTOR.frames.length} canonical frames`)
  }
  return {
    entries,
    animations: DESCRIPTOR.animations,
    aliases: DESCRIPTOR.aliases,
    compositions: DESCRIPTOR.compositions,
  }
}
