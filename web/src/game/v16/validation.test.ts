import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import gameplayV16Schema from '../../../../api/gameplay-v16.schema.json'
import clientFixtures from '../../../../api/fixtures/gameplay-v16-client-conformance.json'
import { snapshot } from '../client/testSupport'
import type { Snapshot } from './schema'
import { validateGameplayV16ServerMessage } from './validation'

function v16Snapshot(enabled = false): Snapshot {
  const message = structuredClone(snapshot(2)) as unknown as Snapshot
  message.room.rulesVersion = 8
  const lairs = { enabled, spawnSeconds: 5, maxLivingMobs: 2 }
  message.room.configuration.environment = { stormCloud: false, lairs }
  message.room.settings.environment = { stormCloud: false, lairs: { ...lairs } }
  message.state.physicalTick = message.room.tick
  message.state.stormCloud = null
  message.state.lairs = enabled ? [
    { id: 0, cell: { column: 6, row: 6 }, spawnRemainingTicks: 300, livingMobs: 0 },
    { id: 1, cell: { column: 8, row: 6 }, spawnRemainingTicks: 300, livingMobs: 0 },
  ] : []
  message.state.mobs = []
  if (enabled) {
    message.state.arena.terrain[6 * 15 + 6] = 'wall'
    message.state.arena.terrain[6 * 15 + 8] = 'wall'
  }
  for (const actor of message.state.actors) {
    if (actor.type === 'visible') actor.stormSlowRemainingTicks = 0
  }
  return message
}

describe('gameplay v16 lair validation', () => {
  it('compiles the strict client and server schema', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV16Schema)
    const client = ajv.getSchema(`${gameplayV16Schema.$id}#/$defs/GameClientMessage`)
    expect(client).toBeTypeOf('function')
    expect(ajv.getSchema(`${gameplayV16Schema.$id}#/$defs/GameServerMessage`)).toBeTypeOf('function')
    for (const fixture of clientFixtures.validMessages) expect(client!(fixture.value)).toBe(true)
    for (const fixture of clientFixtures.invalidMessages) expect(client!(fixture.value)).toBe(false)
  })

  it('accepts disabled and enabled canonical states', () => {
    expect(validateGameplayV16ServerMessage(v16Snapshot())).toMatchObject({ valid: true })
    expect(validateGameplayV16ServerMessage(v16Snapshot(true))).toMatchObject({ valid: true })
  })

  it.each([
    ['environment mismatch', (message: Snapshot) => { message.room.settings.environment.lairs.spawnSeconds = 6 }],
    ['nonempty disabled state', (message: Snapshot) => { message.state.lairs.push({ id: 0, cell: { column: 6, row: 6 }, spawnRemainingTicks: 0, livingMobs: 0 }) }],
    ['wrong layout', (message: Snapshot) => { message.state.lairs[0]!.cell.column = 5 }],
    ['timer overflow', (message: Snapshot) => { message.state.lairs[0]!.spawnRemainingTicks = 301 }],
    ['count mismatch', (message: Snapshot) => { message.state.lairs[0]!.livingMobs = 1 }],
  ] as const)('rejects %s', (_label, mutate) => {
    const message = v16Snapshot(_label === 'nonempty disabled state' ? false : true)
    mutate(message)
    expect(validateGameplayV16ServerMessage(message)).toMatchObject({ valid: false })
  })

  it('rejects actor, bomb, and mob id collisions and non-global frozen state', () => {
    const message = v16Snapshot(true)
    const actorId = message.state.actors[0]!.id
    message.state.mobs = [
      { id: actorId, lairId: 0, position: { x: 7168, y: 6656 }, heading: 'east', frozen: false },
    ]
    message.state.lairs[0]!.livingMobs = 1
    expect(validateGameplayV16ServerMessage(message)).toMatchObject({ valid: false })

    message.state.mobs[0]!.id = 100
    message.state.mobs.push({ id: 101, lairId: 1, position: { x: 8704, y: 6656 }, heading: 'west', frozen: true })
    message.state.lairs[1]!.livingMobs = 1
    expect(validateGameplayV16ServerMessage(message)).toMatchObject({ valid: false })
  })
})
