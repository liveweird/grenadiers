import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV17Schema from '../../../../api/gameplay-v17.schema.json'
import clientFixtures from '../../../../api/fixtures/gameplay-v17-client-conformance.json'
import { snapshot } from '../client/testSupport'
import { validateGameplayV16ServerMessage } from '../v16/validation'
import type { Snapshot, VisibleActorView } from './schema'
import { validateGameplayV17ServerMessage } from './validation'

function frame(): Snapshot {
  return structuredClone(snapshot(2))
}

function actor(value: Snapshot, index: number): VisibleActorView {
  const candidate = value.state.actors[index]
  if (candidate?.type !== 'visible') throw new Error('expected visible actor')
  return candidate
}

describe('gameplay v17 native validation', () => {
  it('compiles the v17 schema and accepts only the rules9 cap-one client corpus', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV17Schema)
    const client = ajv.getSchema(`${gameplayV17Schema.$id}#/$defs/GameClientMessage`)
    expect(client).toBeTypeOf('function')
    for (const fixture of clientFixtures.validMessages) expect(client!(fixture.value), fixture.caseId).toBe(true)
    for (const fixture of clientFixtures.invalidMessages) expect(client!(fixture.value), fixture.caseId).toBe(false)
  })

  it('accepts independent empty-lair countdowns including eligible zero', () => {
    const value = frame()
    value.state.lairs[0]!.spawnRemainingTicks = 0
    value.state.lairs[1]!.spawnRemainingTicks = 419
    expect(validateGameplayV17ServerMessage(value)).toEqual({ valid: true, value })
  })

  it('requires null while occupied and a countdown while empty', () => {
    const occupied = frame()
    occupied.state.mobs.push({
      id: 10, lairId: 0, position: { x: 5_632, y: 6_656 }, heading: 'west', frozen: false,
    })
    occupied.state.lairs[0]!.livingMobs = 1
    occupied.state.lairs[0]!.spawnRemainingTicks = null
    expect(validateGameplayV17ServerMessage(occupied)).toEqual({ valid: true, value: occupied })

    occupied.state.lairs[0]!.spawnRemainingTicks = 0
    expect(validateGameplayV17ServerMessage(occupied)).toMatchObject({ valid: false })
    const empty = frame()
    empty.state.lairs[0]!.spawnRemainingTicks = null
    expect(validateGameplayV17ServerMessage(empty)).toMatchObject({ valid: false })
  })

  it('rejects more than two mobs before semantic lair accounting', () => {
    const value = frame()
    value.state.mobs = [0, 1, 2].map((id) => ({
      id: id + 10, lairId: id % 2, position: { x: 4_000 + id * 700, y: 4_000 },
      heading: 'east' as const, frozen: false,
    }))
    expect(validateGameplayV17ServerMessage(value)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('rejects same-cell and positive-area living-player overlap but permits edge touch', () => {
    const sameCell = frame()
    actor(sameCell, 1).position = { x: 700, y: 700 }
    expect(validateGameplayV17ServerMessage(sameCell)).toMatchObject({ valid: false })

    const overlap = frame()
    actor(overlap, 0).position = { x: 2_000, y: 2_000 }
    actor(overlap, 1).position = { x: 2_639, y: 2_639 }
    expect(validateGameplayV17ServerMessage(overlap)).toMatchObject({ valid: false })

    actor(overlap, 1).position = { x: 2_640, y: 2_000 }
    expect(validateGameplayV17ServerMessage(overlap)).toEqual({ valid: true, value: overlap })

    actor(overlap, 1).position = { ...actor(overlap, 0).position }
    actor(overlap, 1).alive = false
    expect(validateGameplayV17ServerMessage(overlap)).toEqual({ valid: true, value: overlap })
  })

  it('does not invent positions for hidden living actors', () => {
    const value = frame()
    const hidden = actor(value, 1)
    value.state.actors[1] = { type: 'hidden', id: hidden.id, seat: hidden.seat, alive: true }
    expect(validateGameplayV17ServerMessage(value)).toEqual({ valid: true, value })
  })

  it('cross-rejects v16 and v17 protocol identities', () => {
    expect(validateGameplayV17ServerMessage({ type: 'authenticated', protocolVersion: 16 }))
      .toMatchObject({ valid: false, source: 'schema' })
    expect(validateGameplayV17ServerMessage({ type: 'authenticated', protocolVersion: 17 }))
      .toEqual({ valid: true, value: { type: 'authenticated', protocolVersion: 17 } })
    expect(validateGameplayV16ServerMessage({ type: 'authenticated', protocolVersion: 17 }))
      .toMatchObject({ valid: false, source: 'schema' })
  })
})
