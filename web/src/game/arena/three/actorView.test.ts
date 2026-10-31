import { describe, expect, it } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import { createSourceManifest } from '../artAtlas/manifest'
import { RENDERABLE_PICKUP_KINDS } from '../types'
import { actorCues, actorFrameId, createActorView, cueOpacity } from './actorView'
import { bombView, pickupFrame } from './entityView'

describe('engine-independent entity views', () => {
  it('maps every actor appearance, direction, and walking pose to canonical frames', () => {
    const snapshot = gameSnapshot()
    const actor = snapshot.state.actors[0]
    const frameIds = new Set(createSourceManifest().frames.map(({ id }) => id))
    for (const seat of ['one', 'two', 'three', 'four'] as const) {
      for (const heading of ['north', 'east', 'south', 'west'] as const) {
        Object.assign(actor, { seat, heading })
        const view = createActorView(actor, true)
        for (const now of [0, 110, 220, 330]) {
          expect(frameIds.has(actorFrameId(view, now, true))).toBe(true)
        }
      }
    }
  })

  it('projects all public statuses and equipment into existing atlas cues', () => {
    const actor = gameSnapshot().state.actors[0]
    Object.assign(actor, {
      damageGraceRemainingTicks: 1, shieldRemainingTicks: 1, stopwatchRemainingTicks: 1,
      voluntaryActionsFrozen: true, invisibilityCloakRemainingTicks: 1,
      goldenFlameRemainingTicks: 1, shortFuseRemainingTicks: 1, diarrheaRemainingTicks: 1,
      slowFootRemainingTicks: [1, 1, 1, 1, 1, 1, 1], stormSlowRemainingTicks: 1,
      shortFlameRemainingTicks: 1, longFuseRemainingTicks: 1,
      reverseControlsRemainingTicks: 1, invisibleBombsRemainingTicks: 1,
      piercingArmed: true, powerArmed: true, landmineStored: true, remoteArmed: true,
      kickEquipped: true, punchEquipped: true, braceletEquipped: true, wallPassEquipped: true,
    })
    const cues = actorCues(actor)
    const frameIds = new Set(createSourceManifest().frames.map(({ id }) => id))
    const aliases = createSourceManifest().aliases
    expect(cues.map(({ frameId }) => frameId)).toEqual(expect.arrayContaining([
      'actor.cue.protection', 'actor.cue.shield', 'actor.cue.stopwatch', 'actor.cue.frozen',
      'actor.cue.goldenFlame', 'actor.cue.shortFuse', 'actor.cue.shortFlame',
      'actor.cue.longFuse', 'actor.cue.reverseControls', 'actor.cue.diarrhea',
      'actor.cue.slowBoot', 'actor.cue.slowMarker.6', 'actor.cue.piercing',
      'actor.cue.power', 'actor.cue.landmine', 'actor.cue.kick', 'actor.cue.punch',
      'actor.cue.bracelet', 'actor.cue.wallPass', 'actor.cue.cloak',
      'pickup.invisibleBombs', 'bomb.shell.remote.standard',
    ]))
    expect(cues.every(({ frameId }) => frameIds.has(aliases[frameId] ?? frameId))).toBe(true)
    expect(cueOpacity(cues[0], 0, true)).toBeCloseTo(0.65)
    expect(cueOpacity(cues[0], 120, true)).toBe(1)
    expect(cueOpacity(cues[0], 120, false)).toBe(1)
  })

  it('covers every pickup and selects bomb variants from snapshot countdown state', () => {
    const frameIds = new Set(createSourceManifest().frames.map(({ id }) => id))
    expect(RENDERABLE_PICKUP_KINDS.every((kind) => frameIds.has(pickupFrame(kind)))).toBe(true)
    const snapshot = gameSnapshot()
    const bomb = snapshot.state.bombs[0]
    bomb.blast = 'power'
    bomb.trigger = { type: 'timed', remainingTicks: 0, paused: false }
    expect(bombView(bomb, true)).toMatchObject({
      shellFrame: 'bomb.shell.timed.power.2', shellScale: 1.125, spark: true, paused: false,
    })
    bomb.trigger = { type: 'timed', remainingTicks: 180, paused: false }
    expect(bombView(bomb, true).shellScale).toBeCloseTo(1.125)
    bomb.trigger = { type: 'timed', remainingTicks: 48, paused: false }
    expect(bombView(bomb, true).shellScale).toBeCloseTo(1.125)
    bomb.trigger = { type: 'contact' }
    expect(bombView(bomb, true)).toMatchObject({
      shellFrame: 'bomb.shell.contact.power', shellScale: 1, spark: false, paused: false,
    })
    delete bomb.trigger
    delete bomb.blast
    expect(bombView(bomb, true)).toMatchObject({
      shellFrame: 'bomb.shell.timed.standard', shellScale: 1, spark: true, paused: false,
    })
  })
})
