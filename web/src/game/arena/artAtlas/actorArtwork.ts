import { Container } from 'pixi.js'
import { assembleActorDrawing, type ActorDrawing } from '../actorArt'
import type { Direction, Seat } from '../types'
import type { AtlasRegions } from './regions'

export function createAtlasActor(regions: AtlasRegions, seat: Seat): ActorDrawing {
  const sprite = (id: string) => regions.createSprite(id)
  const slowBoot = sprite('actor.cue.slowBoot')
  const slowMarkers = Array.from({ length: 6 }, (_, index) =>
    sprite(`actor.cue.slowMarker.${index + 1}`))
  const slowFoot = new Container()
  slowFoot.addChild(slowBoot, ...slowMarkers)

  const kick = sprite('actor.cue.kick')
  const punch = sprite('actor.cue.punch')
  const bracelet = sprite('actor.cue.bracelet')
  const equipment = new Container()
  equipment.addChild(kick, punch, bracelet)

  const pose = (direction: Direction): ActorDrawing['poses'][Direction] => [
    sprite(`actor.${seat}.${direction}.0`), sprite(`actor.${seat}.${direction}.1`),
  ]
  const poses: ActorDrawing['poses'] = {
    north: pose('north'), east: pose('east'), south: pose('south'), west: pose('west'),
  }

  return assembleActorDrawing({
    root: new Container(),
    shadow: sprite('actor.shadow'),
    protection: sprite('actor.cue.protection'),
    shield: sprite('actor.cue.shield'),
    stopwatch: sprite('actor.cue.stopwatch'),
    frozen: sprite('actor.cue.frozen'),
    goldenFlame: sprite('actor.cue.goldenFlame'),
    shortFuse: sprite('actor.cue.shortFuse'),
    shortFlame: sprite('actor.cue.shortFlame'),
    longFuse: sprite('actor.cue.longFuse'),
    reverseControls: sprite('actor.cue.reverseControls'),
    diarrhea: sprite('actor.cue.diarrhea'),
    slowFoot, slowBoot, slowMarkers,
    piercing: sprite('actor.cue.piercing'),
    power: sprite('actor.cue.power'),
    landmine: sprite('actor.cue.landmine'),
    equipment, kick, punch, bracelet,
    wallPass: sprite('actor.cue.wallPass'),
    cloak: sprite('actor.cue.cloak'),
    poses,
  })
}
