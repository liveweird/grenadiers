import { Container } from 'pixi.js'
import {
  assembleBombDrawing, assembleFlameDrawing, assembleOilBarrelDrawing,
  type BombDrawing, type FlameDrawing, type OilBarrelDrawing,
} from '../entityArt'
import type { ArenaArtwork, TerrainDrawing } from '../artwork'
import { deterministicVariant } from '../pixel'
import type { RenderablePickupKind, SnapshotMessage } from '../types'
import { createAtlasActor } from './actorArtwork'
import type { AtlasRegions } from './regions'

const TILE = 48

function removeOwnedNodes(root: Container): void {
  for (const child of root.removeChildren()) child.destroy({ children: true })
}

function createTerrain(regions: AtlasRegions): TerrainDrawing {
  const root = new Container()
  return {
    root,
    paint(arena: SnapshotMessage['state']['arena']): void {
      removeOwnedNodes(root)
      arena.terrain.forEach((tile, index) => {
        const column = index % arena.width
        const row = Math.floor(index / arena.width)
        const variant = tile === 'wall'
          ? (deterministicVariant(column, row, 3) === 0 ? 1 : 0)
          : deterministicVariant(column, row, 4) + ((column * 5 + row * 3) % 7 === 0 ? 4 : 0)
        const sprite = regions.createSprite(`terrain.${tile}.${variant}`)
        sprite.position.set(column * TILE, row * TILE)
        root.addChild(sprite)
      })
    },
    clear(): void { removeOwnedNodes(root) },
  }
}

function createBomb(
  regions: AtlasRegions,
  blast: 'standard' | 'piercing' | 'power' = 'standard',
  trigger: 'timed' | 'remote' | 'contact' = 'timed',
): BombDrawing {
  return assembleBombDrawing({
    root: new Container(),
    shadow: regions.createSprite('bomb.shadow'),
    shell: regions.createSprite(`bomb.shell.${trigger}.${blast}`),
    spark: regions.createSprite('bomb.spark'),
    pause: regions.createSprite('bomb.pause'),
  }, trigger)
}

function createFlame(regions: AtlasRegions): FlameDrawing {
  return assembleFlameDrawing(new Container(), [
    regions.createSprite('effect.flame.0'), regions.createSprite('effect.flame.1'),
  ])
}

function createOilBarrel(regions: AtlasRegions): OilBarrelDrawing {
  return assembleOilBarrelDrawing(
    new Container(), regions.createSprite('oil.unlit'), regions.createSprite('oil.charred'),
    createFlame(regions),
  )
}

export function createAtlasArtwork(regions: AtlasRegions): ArenaArtwork {
  return {
    makeTerrain: () => createTerrain(regions),
    makeBlock: () => regions.createSprite('terrain.block.brick'),
    makePickup: (kind: RenderablePickupKind) => regions.createSprite(`pickup.${kind}`),
    makeActor: (seat) => createAtlasActor(regions, seat),
    makeBomb: (blast, trigger) => createBomb(regions, blast, trigger),
    makeFlame: () => createFlame(regions),
    makeOilBarrel: () => createOilBarrel(regions),
  }
}
