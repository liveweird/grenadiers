import { Graphics } from 'pixi.js'
import type { ArenaArtwork, TerrainDrawing } from './artwork'
import {
  makeActor, makeBlock, makeBomb, makeFlame, makeOilBarrel, makePickup, paintTerrain,
} from './drawing'
import { makeStormCloud } from './weatherArt'
import { makeLair, makeMob } from './lairArt'
import { Container } from 'pixi.js'

function createMob() {
  const poses = {
    north: makeMob('north'), east: makeMob('east'), south: makeMob('south'), west: makeMob('west'),
  }
  const root = new Container()
  root.addChild(poses.north, poses.east, poses.south, poses.west)
  return { root, poses }
}

function makeTerrain(): TerrainDrawing {
  const root = new Graphics()
  return {
    root,
    paint: (arena) => paintTerrain(root, arena),
    clear: () => { root.clear() },
  }
}

export const graphicsArtwork: ArenaArtwork = {
  makeTerrain,
  makeBlock,
  makePickup,
  makeActor,
  makeBomb,
  makeFlame,
  makeOilBarrel,
  makeStormCloud,
  makeLair,
  makeMob: createMob,
}
