import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import clientFixtures from '../../../../api/fixtures/gameplay-v18-client-conformance.json'
import serverFixtures from '../../../../api/fixtures/gameplay-v18-server-conformance.json'
import gameplayV18Schema from '../../../../api/gameplay-v18.schema.json'
import validateServer from '../../validation/generated/gameplay-v18.mjs'
import { validatorOptions } from '../../validation/validatorOptions'
import type { Snapshot } from './schema'

type Validator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }

function errors(validator: Validator): ErrorObject[] | null {
  return structuredClone(validator.errors ?? null)
}

describe('gameplay v18 contract validation', () => {
  it('accepts only the bounded strict spike-pit client corpus', () => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV18Schema)
    const validateClient = ajv.getSchema(`${gameplayV18Schema.$id}#/$defs/GameClientMessage`)
    expect(validateClient).toBeTypeOf('function')
    for (const fixture of clientFixtures.validMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(true)
    }
    for (const fixture of clientFixtures.invalidMessages) {
      expect(validateClient!(fixture.value), fixture.caseId).toBe(false)
    }
  })

  it('accepts every Rust-generated recipient frame through the standalone validator', () => {
    const original = new Ajv(validatorOptions('gameplay-v18')).addSchema(gameplayV18Schema)
      .getSchema(`${gameplayV18Schema.$id}#/$defs/GameServerMessage`) as ValidateFunction
    expect(serverFixtures.cases.map(({ caseId }) => caseId)).toEqual([
      'lobby-room-state', 'waiting-admission', 'disabled', 'enabled-empty',
      'idle', 'warning', 'raised',
    ])
    for (const fixture of serverFixtures.cases) {
      const frame: unknown = JSON.parse(fixture.frame)
      expect(original(frame), fixture.caseId).toBe(true)
      expect(validateServer(frame), fixture.caseId).toBe(true)
      expect(errors(validateServer)).toEqual(errors(original))
    }
    for (const malformed of [
      { type: 'authenticated' },
      { type: 'authenticated', protocolVersion: 17 },
      { type: 'authenticated', protocolVersion: 18, unexpected: true },
      { type: 'nonsense' },
    ]) {
      expect(validateServer(malformed)).toBe(original(malformed))
      expect(errors(validateServer)).toEqual(errors(original))
    }
    const invalid = { type: 'authenticated', protocolVersion: 17 }
    expect(validateServer(invalid)).toBe(original(invalid))
    const retainedGenerated = validateServer.errors
    const retainedOriginal = original.errors
    expect(validateServer({ type: 'authenticated', protocolVersion: 18 })).toBe(true)
    expect(original({ type: 'authenticated', protocolVersion: 18 })).toBe(true)
    expect(validateServer.errors).toBeNull()
    expect(original.errors).toBeNull()
    expect(validateServer(invalid)).toBe(original(invalid))
    expect(errors(validateServer)).toEqual(errors(original))
    expect(validateServer.errors).not.toBe(retainedGenerated)
    expect(original.errors).not.toBe(retainedOriginal)
  })

  it('projects only public symmetric pit facts with configuration-bound clocks', () => {
    const snapshots: Array<readonly [string, Snapshot]> = []
    for (const { caseId, frame: encoded } of serverFixtures.cases) {
      const frame: unknown = JSON.parse(encoded)
      if (!validateServer(frame)) throw new Error(`invalid generated fixture: ${caseId}`)
      if (frame.type === 'snapshot') snapshots.push([caseId, frame])
    }
    for (const [caseId, frame] of snapshots) {
      const configured = frame.room.configuration.environment.spikePits
      expect(frame.room.settings.environment.spikePits, caseId).toEqual(configured)
      const pits = frame.state.spikePits
      expect([0, 2, 4], caseId).toContain(pits.length)
      expect(pits.map(({ cell }: { cell: { column: number; row: number } }) =>
        `${cell.row},${cell.column}`), caseId).toEqual(
        [...pits].sort((left, right) => left.cell.row - right.cell.row
          || left.cell.column - right.cell.column)
          .map(({ cell }) => `${cell.row},${cell.column}`),
      )
      expect(fixturePrivateKeys(frame), caseId).toEqual([])
    }
    const byCase = Object.fromEntries(snapshots)
    expect(byCase.disabled.state.spikePits).toEqual([])
    expect(byCase['enabled-empty'].state.spikePits).toEqual([])
    expect(byCase.warning.state.spikePits[0]).toMatchObject({ phase: 'warning', remainingTicks: 6 })
    expect(byCase.raised.state.spikePits[0]).toMatchObject({ phase: 'raised', remainingTicks: 60 })
  })

  it.each([
    ['idle', 0, true],
    ['idle', 1, false],
    ['warning', 1, true],
    ['warning', 121, false],
    ['raised', 600, true],
    ['raised', 0, false],
  ])('enforces the %s remaining-tick shape at %i', (phase, remainingTicks, valid) => {
    const ajv = new Ajv({ allErrors: true }).addSchema(gameplayV18Schema)
    const validatePit = ajv.getSchema(`${gameplayV18Schema.$id}#/$defs/SpikePitView`)
    expect(validatePit!({ cell: { column: 3, row: 3 }, phase, remainingTicks })).toBe(valid)
  })
})

function fixturePrivateKeys(value: unknown): string[] {
  const forbidden = new Set(['activator', 'seed', 'rng', 'route', 'target'])
  const found = new Set<string>()
  const visit = (candidate: unknown): void => {
    if (Array.isArray(candidate)) {
      candidate.forEach(visit)
    } else if (candidate !== null && typeof candidate === 'object') {
      for (const [key, nested] of Object.entries(candidate)) {
        if (forbidden.has(key)) found.add(key)
        visit(nested)
      }
    }
  }
  visit(value)
  return [...found].sort()
}
