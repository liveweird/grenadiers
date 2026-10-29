import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV15Schema from '../../../../api/gameplay-v15.schema.json'
import clientFixtures from '../../../../api/fixtures/gameplay-v15-client-conformance.json'
import { snapshot } from '../client/testSupport'
import type { Snapshot } from './schema'
import { validateGameplayV15ServerMessage } from './validation'
import { validateGameplayV14ServerMessage } from '../v14/validation'

function v15Snapshot(stormCloud = true): Snapshot {
  const message = structuredClone(snapshot(2)) as unknown as Snapshot
  message.room.rulesVersion = 7
  message.room.configuration.environment = { stormCloud }
  message.room.settings.environment = { stormCloud }
  message.state.stormCloud = stormCloud ? {
    position: { x: 7_680, y: 6_656 }, direction: 'east', legRemainingTicks: 128,
  } : null
  for (const actor of message.state.actors) {
    if (actor.type === 'visible') actor.stormSlowRemainingTicks = 0
  }
  return message
}

describe('gameplay v15 storm environment validation', () => {
  it('keeps browser client-schema admission aligned with the Rust parser corpus', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV15Schema)
    const validateClient = ajv.getSchema(`${gameplayV15Schema.$id}#/$defs/GameClientMessage`)
    if (!validateClient) throw new Error('missing gameplay v15 client validator')
    for (const fixture of clientFixtures.validMessages) expect(validateClient(fixture.value)).toBe(true)
    for (const fixture of clientFixtures.invalidMessages) expect(validateClient(fixture.value)).toBe(false)
  })

  it('accepts the rules-7 single-cloud projection', () => {
    const message = v15Snapshot()
    expect(validateGameplayV15ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('keeps the closed v14 validator strict against v15 fields and rules7', () => {
    expect(validateGameplayV14ServerMessage(v15Snapshot())).toMatchObject({
      valid: false, source: 'schema',
    })
  })

  it('accepts the rules-7 disabled projection with null cloud and zero timers', () => {
    const message = v15Snapshot(false)
    expect(validateGameplayV15ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([
    ['configuration mismatch', (message: Snapshot) => { message.room.configuration.environment.stormCloud = false }],
    ['missing enabled cloud', (message: Snapshot) => { message.state.stormCloud = null }],
    ['disabled cloud', (message: Snapshot) => { message.room.configuration.environment.stormCloud = false; message.room.settings.environment.stormCloud = false }],
    ['outside cloud', (message: Snapshot) => { message.state.stormCloud!.position.x = 512 }],
    ['inconsistent leg', (message: Snapshot) => { message.state.stormCloud!.legRemainingTicks = 127 }],
  ] as const)('rejects %s facts', (_label, mutate) => {
    const message = v15Snapshot()
    mutate(message)
    expect(validateGameplayV15ServerMessage(message)).toMatchObject({ valid: false })
  })

  it('rejects a nonzero timer while the environment is disabled', () => {
    const message = v15Snapshot(false)
    const actor = message.state.actors[0]!
    if (actor.type !== 'visible') throw new Error('fixture actor must be visible')
    actor.stormSlowRemainingTicks = 1
    expect(validateGameplayV15ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidStormEnvironment',
    })
  })

  it('rejects a timer retained by a dead visible actor', () => {
    const message = v15Snapshot()
    const actor = message.state.actors[0]!
    if (actor.type !== 'visible') throw new Error('fixture actor must be visible')
    actor.alive = false
    actor.stormSlowRemainingTicks = 1
    expect(validateGameplayV15ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidStormEnvironment',
    })
  })

  it('does not permit a hidden actor timer privacy leak', () => {
    const message = v15Snapshot()
    const actor = message.state.actors[1]!
    message.state.actors[1] = {
      type: 'hidden', id: actor.id, seat: actor.seat, alive: actor.alive,
      stormSlowRemainingTicks: 12,
    } as unknown as Snapshot['state']['actors'][number]
    expect(validateGameplayV15ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })

  it.each([
    ['timer overflow', (message: Snapshot) => {
      const actor = message.state.actors[0]!
      if (actor.type === 'visible') actor.stormSlowRemainingTicks = 181
    }],
    ['leg overflow', (message: Snapshot) => { message.state.stormCloud!.legRemainingTicks = 129 }],
    ['extra environment field', (message: Snapshot) => {
      Object.assign(message.room.settings.environment, { seed: 7 })
    }],
  ] as const)('schema-rejects %s', (_label, mutate) => {
    const message = v15Snapshot()
    mutate(message)
    expect(validateGameplayV15ServerMessage(message)).toMatchObject({ valid: false, source: 'schema' })
  })
})
