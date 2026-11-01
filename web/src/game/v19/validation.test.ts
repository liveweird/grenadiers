import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import clientFixtures from '../../../../api/fixtures/gameplay-v19-client-conformance.json'
import serverFixtures from '../../../../api/fixtures/gameplay-v19-server-conformance.json'
import gameplayV19Schema from '../../../../api/gameplay-v19.schema.json'
import validateServer from '../../validation/generated/gameplay-v19.mjs'
import { validatorOptions } from '../../validation/validatorOptions'
import { snapshot } from '../client/testSupport'
import type { Snapshot } from './schema'
import { validateGameplayV19ServerMessage } from './validation'

type Validator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }
const errors = (validator: Validator): ErrorObject[] | null => structuredClone(validator.errors ?? null)

function hardSnapshot(): Snapshot {
  const frame: Snapshot = JSON.parse(JSON.stringify(snapshot(10)))
  frame.room.configuration = {
    ...frame.room.configuration, controllers: ['human', 'cpu'], cpuDifficulty: 'hard',
  }
  frame.room.seats[1] = {
    seat: 'two', occupied: true, ready: false, rematchReady: false,
    controller: { type: 'cpu', difficulty: 'hard' }, connection: { type: 'serverControlled' },
  }
  return frame
}

describe('gameplay v19 contract validation', () => {
  it('accepts the closed client corpus and rejects every negative case', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV19Schema)
    const validateClient = ajv.getSchema(`${gameplayV19Schema.$id}#/$defs/GameClientMessage`)
    for (const fixture of clientFixtures.validMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(true)
    }
    for (const fixture of clientFixtures.invalidMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(false)
    }
  })

  it('keeps standalone validation in canonical Ajv parity', () => {
    const original = new Ajv(validatorOptions('gameplay-v19')).addSchema(gameplayV19Schema)
      .getSchema(`${gameplayV19Schema.$id}#/$defs/GameServerMessage`) as ValidateFunction
    for (const fixture of [...serverFixtures.validMessages, ...serverFixtures.invalidMessages]) {
      expect(validateServer(fixture.value), fixture.caseId).toBe(original(fixture.value))
      expect(errors(validateServer)).toEqual(errors(original))
    }
  })

  it('enforces the closed Hard, null, extra-field, and privacy fixture boundary', () => {
    for (const fixture of serverFixtures.validMessages) {
      expect(validateGameplayV19ServerMessage(fixture.value), fixture.caseId)
        .toMatchObject({ valid: true })
    }
    const expectedSource: Record<string, 'schema' | 'semantics'> = {
      'mixed-cpu-difficulties': 'semantics',
      'cpu-room-null-difficulty': 'schema',
      'extra-room-planner-field': 'schema',
      'private-cpu-route': 'schema',
      'v18-identity': 'schema',
    }
    for (const fixture of serverFixtures.invalidMessages) {
      expect(validateGameplayV19ServerMessage(fixture.value), fixture.caseId)
        .toMatchObject({ valid: false, source: expectedSource[fixture.caseId] })
    }
  })

  it('accepts Hard only when every CPU seat matches the room difficulty', () => {
    const frame = hardSnapshot()
    expect(validateGameplayV19ServerMessage(frame)).toMatchObject({ valid: true })
    frame.room.seats[1]!.controller = { type: 'cpu', difficulty: 'standard' }
    expect(validateGameplayV19ServerMessage(frame)).toEqual({
      valid: false, source: 'semantics', error: 'invalidCpuDifficulty',
    })
  })

  it.each([
    ['extra seat status', 'invalidConfiguration', (frame: Snapshot) => {
      frame.room.seats.push({
        seat: 'three', occupied: false, ready: false, rematchReady: false,
        controller: { type: 'human' }, connection: { type: 'vacant' },
      })
    }],
    ['duplicate score seat', 'invalidConfiguration', (frame: Snapshot) => {
      frame.room.scores[1]!.seat = 'one'
    }],
    ['misordered score seats', 'invalidConfiguration', (frame: Snapshot) => {
      frame.room.scores.reverse()
    }],
    ['misordered actors', 'invalidRoster', (frame: Snapshot) => {
      frame.state.actors.reverse()
    }],
    ['invalid lifecycle', 'invalidLifecycle', (frame: Snapshot) => {
      frame.room.roundNumber = 2
    }],
    ['invalid world binding', 'invalidStateBinding', (frame: Snapshot) => {
      frame.state.physicalTick += 1
    }],
    ['leaked bomb count', 'invalidWorldState', (frame: Snapshot) => {
      const actor = frame.state.actors[0]
      if (actor?.type === 'visible') actor.liveBombCount = 1
    }],
    ['invalid lair facts', 'invalidLairState', (frame: Snapshot) => {
      frame.state.lairs[0]!.livingMobs = 1
    }],
    ['invalid pit cardinality', 'invalidSpikePitState', (frame: Snapshot) => {
      frame.state.spikePits.push({
        cell: { column: 3, row: 3 }, phase: 'idle', remainingTicks: 0,
      })
    }],
  ])('rejects schema-valid rules-10 semantics: %s', (_name, error, mutate) => {
    const frame = hardSnapshot()
    mutate(frame)
    expect(validateServer(frame)).toBe(true)
    expect(validateGameplayV19ServerMessage(frame)).toEqual({
      valid: false, source: 'semantics', error,
    })
  })

  it('rejects private CPU planning data at the closed schema boundary', () => {
    const frame: unknown = JSON.parse(JSON.stringify(hardSnapshot()))
    const record = frame as { room: { seats: Array<{ controller: Record<string, unknown> }> } }
    record.room.seats[1]!.controller.route = [{ column: 2, row: 1 }]
    expect(validateGameplayV19ServerMessage(frame)).toMatchObject({ valid: false, source: 'schema' })
  })
})
