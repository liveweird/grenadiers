import type { Container } from 'pixi.js'
import type { ActorDrawing } from './actorArt'
import type { BombDrawing, FlameDrawing, OilBarrelDrawing } from './entityArt'
import type { RenderablePickupKind, Seat, SnapshotMessage } from './types'

export interface TerrainDrawing {
  root: Container
  paint(arena: SnapshotMessage['state']['arena']): void
  clear(): void
}

export interface ArenaArtwork {
  makeTerrain(): TerrainDrawing
  makeBlock(): Container
  makePickup(kind: RenderablePickupKind): Container
  makeActor(seat: Seat): ActorDrawing
  makeBomb(
    blast?: 'standard' | 'piercing' | 'power',
    trigger?: 'timed' | 'remote' | 'contact',
  ): BombDrawing
  makeFlame(): FlameDrawing
  makeOilBarrel(): OilBarrelDrawing
}
