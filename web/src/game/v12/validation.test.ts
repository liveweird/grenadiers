import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import serverFixtures from '../../../../api/fixtures/gameplay-expanded-server-conformance.json'
import clientFixtures from '../../../../api/fixtures/gameplay-v12-client-conformance.json'
import gameplayV11Schema from '../../../../api/gameplay-v11.schema.json'
import gameplayV12Schema from '../../../../api/gameplay-v12.schema.json'
import { admitted, snapshot } from '../v13/testSupport'
import type { GameServerMessage } from './schema'
import { validateGameplayV12ServerMessage } from './validation'

type SnapshotMessage = Extract<GameServerMessage, { type: 'snapshot' }>

const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV11Schema).addSchema(gameplayV12Schema)
const v11Client = ajv.getSchema(`${gameplayV11Schema.$id}#/$defs/GameClientMessage`)!
const v12Client = ajv.getSchema(`${gameplayV12Schema.$id}#/$defs/GameClientMessage`)!
const v11Server = ajv.getSchema(`${gameplayV11Schema.$id}#/$defs/GameServerMessage`)!

function duelMessage(): SnapshotMessage {
  const message = snapshot(2) as unknown as SnapshotMessage
  message.room.mode = { type: 'humanDuel' }
  for (const seat of message.room.seats) seat.controller = { type: 'human' }
  return message
}

function soloMessage(): SnapshotMessage {
  const message = duelMessage()
  message.room.mode = { type: 'solo', difficulty: 'standard' }
  message.room.settingsEditor = 'one'
  message.room.seats[1]!.controller = { type: 'cpu', difficulty: 'standard' }
  message.room.seats[1]!.occupied = true
  message.room.seats[1]!.connection = { type: 'serverControlled' }
  return message
}

function withV12Metadata(frame: string, solo: boolean): GameServerMessage {
  const message = JSON.parse(frame) as GameServerMessage
  if (message.type !== 'admitted' && message.type !== 'roomState' && message.type !== 'snapshot') {
    throw new Error('fixture must carry a room')
  }
  message.room.mode = solo
    ? { type: 'solo', difficulty: 'standard' }
    : { type: 'humanDuel' }
  for (const seat of message.room.seats) seat.controller = { type: 'human' }
  if (solo) {
    message.room.settingsEditor = 'one'
    message.room.seats[1]!.controller = { type: 'cpu', difficulty: 'standard' }
    message.room.seats[1]!.connection = { type: 'serverControlled' }
  }
  return message
}

describe('gameplay v12 solo metadata', () => {
  it('accepts only the two closed create-room modes while v11 stays closed', () => {
    for (const fixture of clientFixtures.validCreateRooms) {
      expect(v12Client(fixture.value), fixture.caseId).toBe(true)
      expect(v11Client(fixture.value), fixture.caseId).toBe(false)
    }
    for (const fixture of clientFixtures.invalidCreateRooms) {
      expect(v12Client(fixture.value), fixture.caseId).toBe(false)
    }
  })

  it('accepts human-duel and solo projections without changing v11 physical semantics', () => {
    for (const message of [duelMessage(), soloMessage()]) {
      expect(validateGameplayV12ServerMessage(message)).toEqual({ valid: true, value: message })
      expect(v11Server(message)).toBe(false)
    }
  })

  it('retains v11 waiting, recipient privacy, Oil, and running-state conformance', () => {
    const fixtures = serverFixtures.cases.filter(({ version }) => version === 11)
    for (const fixture of fixtures) {
      const duel = withV12Metadata(fixture.frame, false)
      expect(validateGameplayV12ServerMessage(duel), `${fixture.caseId}-duel`).toMatchObject({
        valid: true,
      })
      expect(v11Server(duel), `${fixture.caseId}-v11-closed`).toBe(false)
      if (fixture.scenario !== 'waitingAdmission') {
        const solo = withV12Metadata(fixture.frame, true)
        expect(validateGameplayV12ServerMessage(solo), `${fixture.caseId}-solo`).toMatchObject({
          valid: true,
        })
      }
    }
  })

  it('rejects cross-mode controller and connection combinations', () => {
    const mutations = [
      (message: ReturnType<typeof soloMessage>) => { message.room.mode = { type: 'humanDuel' } },
      (message: ReturnType<typeof soloMessage>) => { message.room.seats[0]!.controller = { type: 'cpu', difficulty: 'standard' } },
      (message: ReturnType<typeof soloMessage>) => { message.room.seats[0]!.connection = { type: 'serverControlled' } },
      (message: ReturnType<typeof soloMessage>) => { message.room.seats[1]!.controller = { type: 'human' } },
      (message: ReturnType<typeof soloMessage>) => { message.room.seats[1]!.connection = { type: 'connected' } },
      (message: ReturnType<typeof soloMessage>) => { message.room.seats[1]!.occupied = false },
      (message: ReturnType<typeof soloMessage>) => { message.room.settingsEditor = 'two' },
    ]
    for (const mutate of mutations) {
      const message = soloMessage()
      mutate(message)
      expect(validateGameplayV12ServerMessage(message)).toMatchObject({
        valid: false, source: 'semantics', error: 'invalidControllerMetadata',
      })
    }
  })

  it('requires the eligible human owner but permits a checked closing room without one', () => {
    const missingOwner = soloMessage()
    missingOwner.room.settingsEditor = null
    expect(validateGameplayV12ServerMessage(missingOwner)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidControllerMetadata',
    })

    const closing = soloMessage()
    closing.room.settingsEditor = null
    closing.room.seats[0]!.occupied = false
    closing.room.seats[0]!.connection = { type: 'vacant' }
    closing.room.phase = {
      type: 'matchResult', endedAt: closing.room.tick,
      result: { type: 'winner', seat: 'two', reason: 'forfeit' },
      rematchAt: closing.room.tick,
    }
    expect(validateGameplayV12ServerMessage(closing)).toEqual({ valid: true, value: closing })
  })

  it('rejects personal admission to the CPU seat', () => {
    const message = admitted() as unknown as GameServerMessage & { type: 'admitted' }
    const solo = soloMessage() as GameServerMessage & { type: 'snapshot' }
    message.room = solo.room
    message.youSeat = 'two'
    expect(validateGameplayV12ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidPersonalAdmission',
    })
  })
})
