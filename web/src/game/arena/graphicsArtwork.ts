import { Graphics } from 'pixi.js'
import type { ArenaArtwork, TerrainDrawing } from './artwork'
import {
  makeActor, makeBlock, makeBomb, makeFlame, makeOilBarrel, makePickup, paintTerrain,
} from './drawing'

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
}
