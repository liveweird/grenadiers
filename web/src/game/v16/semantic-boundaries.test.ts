import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v16-server-conformance.json'
import type { Snapshot, VisibleActorView } from './schema'
import { validateGameplayV16ServerMessage } from './validation'

function snapshot(id = 'disabled11x9Cpu18-one'): Snapshot {
  const entry = corpus.cases.find(({ caseId }) => caseId === id)
  if (!entry) throw new Error(`Missing Rust snapshot ${id}`)
  const frame = JSON.parse(entry.frame) as Snapshot
  expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: true })
  return frame
}

function own(frame: Snapshot): VisibleActorView {
  const actor = frame.state.actors[0]!
  if (actor.type !== 'visible') throw new Error('Expected visible own actor')
  return actor
}

describe('gameplay v16 authoritative semantic boundaries', () => {
  it.each([
    ['ready vacant human', (frame: Snapshot) => {
      Object.assign(frame.room.seats[0]!, {
        occupied: false, ready: true, connection: { type: 'vacant' },
      })
    }, 'invalidControllerMetadata'],
    ['rematch vote from vacant human', (frame: Snapshot) => {
      Object.assign(frame.room.seats[0]!, {
        occupied: false, ready: false, rematchReady: true, connection: { type: 'vacant' },
      })
    }, 'invalidControllerMetadata'],
    ['human labelled as CPU', (frame: Snapshot) => {
      frame.room.seats[0]!.controller = { type: 'cpu', difficulty: 'standard' }
    }, 'invalidControllerMetadata'],
    ['editing grant transferred to another seat', (frame: Snapshot) => {
      frame.room.settingsEditor = 'two'
    }, 'invalidControllerMetadata'],
    ['Oil while disabled', (frame: Snapshot) => {
      frame.state.oilBarrels.push({ cell: { column: 1, row: 1 }, state: 'unlit' })
    }, 'invalidOilBarrels'],
    ['slow timer while cloud disabled', (frame: Snapshot) => {
      own(frame).stormSlowRemainingTicks = 1
    }, 'invalidStormEnvironment'],
    ['teleport marker while item disabled', (frame: Snapshot) => {
      own(frame).lastTeleportTick = frame.state.physicalTick
    }, 'invalidTeleportMarker'],
    ['teleport marker at physical round start', (frame: Snapshot) => {
      frame.room.settings.enabledItems = ['teleporter']
      own(frame).lastTeleportTick = frame.state.physicalTick - frame.state.runningUpdates
    }, 'invalidTeleportMarker'],
    ['future teleport marker', (frame: Snapshot) => {
      frame.room.settings.enabledItems = ['teleporter']
      own(frame).lastTeleportTick = frame.state.physicalTick + 1
    }, 'invalidTeleportMarker'],
    ['bomb belonging to an unknown actor', (frame: Snapshot) => {
      frame.state.bombs.push({
        id: 777, owner: 999, cell: { column: 1, row: 1 }, range: 2,
        placedAt: frame.state.physicalTick, blast: 'standard',
        trigger: { type: 'timed', remainingTicks: 180, paused: false },
      })
    }, 'invalidWorldState'],
  ] as const)('rejects %s before consumers use the frame', (_label, mutate, error) => {
    const frame = snapshot()
    mutate(frame)
    expect(validateGameplayV16ServerMessage(frame)).toEqual({
      valid: false, source: 'semantics', error,
    })
  })

  it.each(['north', 'west'] as const)('accepts consistent %s cloud motion', (direction) => {
    const frame = snapshot('single13x11Cpu18Timer-one')
    const cloud = frame.state.stormCloud!
    cloud.direction = direction
    cloud.position = direction === 'north' ? { x: 6656, y: 5624 } : { x: 6648, y: 5632 }
    expect(validateGameplayV16ServerMessage(frame)).toEqual({ valid: true, value: frame })
  })
})
