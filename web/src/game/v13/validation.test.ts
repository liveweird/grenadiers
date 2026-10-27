import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import serverFixtures from '../../../../api/fixtures/gameplay-expanded-server-conformance.json'
import clientFixtures from '../../../../api/fixtures/gameplay-v13-client-conformance.json'
import gameplayV12Schema from '../../../../api/gameplay-v12.schema.json'
import gameplayV13Schema from '../../../../api/gameplay-v13.schema.json'
import { snapshot } from '../client/testSupport'
import type { Difficulty, GameServerMessage } from './schema'
import { validateGameplayV13ServerMessage } from './validation'

type SnapshotMessage = Extract<GameServerMessage, { type: 'snapshot' }>

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV12Schema).addSchema(gameplayV13Schema)
const v12Client = ajv.getSchema(`${gameplayV12Schema.$id}#/$defs/GameClientMessage`)!
const v12Server = ajv.getSchema(`${gameplayV12Schema.$id}#/$defs/GameServerMessage`)!
const v13Client = ajv.getSchema(`${gameplayV13Schema.$id}#/$defs/GameClientMessage`)!

function soloMessage(difficulty: Difficulty): SnapshotMessage {
  const message = snapshot(2) as SnapshotMessage
  message.room.mode = { type: 'solo', difficulty }
  message.room.settingsEditor = 'one'
  message.room.seats[1] = {
    seat: 'two', controller: { type: 'cpu', difficulty }, occupied: true,
    ready: true, rematchReady: false, connection: { type: 'serverControlled' },
  }
  return message
}

function withV13Metadata(frame: string, difficulty: Difficulty): GameServerMessage {
  const message = JSON.parse(frame) as GameServerMessage
  if (message.type !== 'admitted' && message.type !== 'roomState' && message.type !== 'snapshot') {
    throw new Error('fixture must carry a room')
  }
  message.room.mode = { type: 'solo', difficulty }
  for (const seat of message.room.seats) seat.controller = { type: 'human' }
  message.room.settingsEditor = 'one'
  message.room.seats[1]!.controller = { type: 'cpu', difficulty }
  message.room.seats[1]!.connection = { type: 'serverControlled' }
  return message
}

describe('gameplay v13 selectable difficulty metadata', () => {
  it('accepts the closed client corpus and rejects authority-bearing inputs', () => {
    for (const fixture of clientFixtures.validCreateRooms) {
      expect(v13Client(fixture.value), fixture.caseId).toBe(true)
    }
    for (const fixture of clientFixtures.invalidCreateRooms) {
      expect(v13Client(fixture.value), fixture.caseId).toBe(false)
    }
  })

  it.each(['easy', 'standard'] as const)('accepts the closed %s create-room mode', (difficulty) => {
    const value = { type: 'createRoom', sequence: 1, mode: { type: 'solo', difficulty } }
    expect(v13Client(value)).toBe(true)
    expect(v12Client(value)).toBe(difficulty === 'standard')
  })

  it.each(['easy', 'standard'] as const)('retains v12 physical semantics for %s metadata', (difficulty) => {
    const message = soloMessage(difficulty)
    expect(validateGameplayV13ServerMessage(message)).toEqual({ valid: true, value: message })
    expect(v12Server(message)).toBe(difficulty === 'standard')
  })

  it.each(['easy', 'standard'] as const)(
    'retains archived waiting, privacy, Oil, and running semantics for %s',
    (difficulty) => {
      const fixtures = serverFixtures.cases.filter(({ version, scenario }) =>
        version === 11 && scenario !== 'waitingAdmission')
      for (const fixture of fixtures) {
        const message = withV13Metadata(fixture.frame, difficulty)
        expect(validateGameplayV13ServerMessage(message), fixture.caseId).toEqual({ valid: true, value: message })
      }
    },
  )

  it('rejects mismatched mode and controller difficulties', () => {
    const message = soloMessage('easy')
    message.room.seats[1]!.controller = { type: 'cpu', difficulty: 'standard' }
    expect(validateGameplayV13ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidControllerMetadata',
    })
  })

  it('rejects unknown fields in both halves of the difficulty metadata pair', () => {
    for (const mutate of [
      (message: SnapshotMessage) => Object.assign(message.room.mode, { cadenceTicks: 12 }),
      (message: SnapshotMessage) => Object.assign(message.room.seats[1]!.controller, { policy: 'slow' }),
    ]) {
      const message = soloMessage('easy')
      mutate(message)
      expect(validateGameplayV13ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
    }
  })

})
