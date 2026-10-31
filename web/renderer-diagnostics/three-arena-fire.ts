import type { SnapshotMessage } from '../src/game/arena/types'
import { threeArenaSnapshot } from './three-arena-snapshots'

export type FireScene =
  | 'empty' | 'single' | 'burningOil' | 'bombOnly' | 'concealed'
  | 'actorOnly' | 'actorNear' | 'transitionBase' | 'transitionAdded' | 'worstCase'
export type FireReset = 'clear' | 'round' | 'admission' | 'backward'

const FIRE_CELL = { column: 5, row: 5 }

function fixtureBase(tick: number): SnapshotMessage {
  const snapshot = threeArenaSnapshot('large', 'empty')
  snapshot.room.phase = { type: 'running' }
  snapshot.room.presentationBaseline = 'three-arena-fire-v1'
  snapshot.state.physicalTick = tick
  return snapshot
}

function addActor(snapshot: SnapshotMessage): void {
  snapshot.state.actors = [{
    id: 10, seat: 'one', alive: true, heading: 'east', lastTeleportTick: null,
    position: { x: 6.5 * 1024, y: 5.5 * 1024 },
  }]
}

export function fireSnapshot(scene: FireScene, tick = 10): SnapshotMessage {
  const snapshot = fixtureBase(tick)
  if (scene === 'single' || scene === 'actorNear' || scene === 'transitionAdded') {
    snapshot.state.flames.push({ cell: FIRE_CELL, expiresAt: 999 })
  }
  if (scene === 'transitionBase' || scene === 'transitionAdded') {
    snapshot.state.flames.unshift({ cell: { column: 3, row: 5 }, expiresAt: 999 })
  }
  if (scene === 'burningOil') {
    snapshot.state.oilBarrels = [{ cell: FIRE_CELL, state: 'burning' }]
  }
  if (scene === 'bombOnly') {
    snapshot.state.bombs = [{
      id: 30, cell: FIRE_CELL, owner: 10, range: 8, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 1, paused: false }, blast: 'power',
    }]
  }
  // A concealed fact has no public row by definition; the snapshot intentionally equals empty.
  if (scene === 'actorOnly' || scene === 'actorNear') addActor(snapshot)
  if (scene === 'worstCase') {
    const { arena } = snapshot.state
    for (let index = 0; index < arena.terrain.length; index += 1) {
      const column = index % arena.width
      const row = Math.floor(index / arena.width)
      const blocked = arena.blocks.some((cell) => cell.column === column && cell.row === row)
      if (arena.terrain[index] === 'floor' && !blocked) {
        snapshot.state.flames.push({ cell: { column, row }, expiresAt: 999 })
      }
    }
  }
  return snapshot
}

export function resetFireSnapshot(reason: FireReset): SnapshotMessage {
  const snapshot = fixtureBase(reason === 'backward' ? 1 : 12)
  if (reason === 'round') snapshot.room.roundId += 1
  if (reason === 'admission') snapshot.room.presentationBaseline = 'three-arena-fire-admission-v2'
  return snapshot
}
