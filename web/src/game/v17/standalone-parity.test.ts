import Ajv, { type ErrorObject, type ValidateFunction } from 'ajv'
import { describe, expect, it } from 'vitest'
import schema from '../../../../api/gameplay-v17.schema.json'
import corpus from '../../../../api/fixtures/gameplay-v17-server-conformance.json'
import generated from '../../validation/generated/gameplay-v17.mjs'

type GeneratedValidator = ((value: unknown) => boolean) & { errors?: ErrorObject[] | null }
const standalone = generated as GeneratedValidator
const original = new Ajv({ allErrors: true }).addSchema(schema)
  .getSchema(`${schema.$id}#/$defs/GameServerMessage`) as ValidateFunction
const frames = corpus.cases.map(({ frame }) => JSON.parse(frame) as Record<string, unknown>)

function errors(validator: { errors?: ErrorObject[] | null }): ErrorObject[] | null {
  return structuredClone(validator.errors ?? null)
}

function expectParity(value: unknown, valid: boolean): void {
  expect(original(value)).toBe(valid)
  expect(standalone(value)).toBe(valid)
  expect(errors(standalone)).toEqual(errors(original))
}

describe('gameplay v17 standalone validator parity', () => {
  it('matches AJV for all 27 authoritative Rust frames', () => {
    for (const [index, frame] of frames.entries()) {
      expectParity(frame, true)
      expect(standalone.errors, corpus.cases[index]!.caseId).toBeNull()
    }
  })

  it('does not relax v17 identity, closure, cap, timer, or total-mob errors', () => {
    const authenticated = { type: 'authenticated', protocolVersion: 17 }
    const snapshot = frames.find(({ type }) => type === 'snapshot')!
    const lairSnapshot = JSON.parse(corpus.cases.find(({ caseId }) =>
      caseId === 'populated11x9Lairs-one')!.frame) as Record<string, unknown>
    const cases: Array<readonly [string, unknown]> = [
      ['protocol identity', { type: 'authenticated', protocolVersion: 16 }],
      ['closed message', { ...authenticated, unexpected: true }],
      ['rules identity', (() => {
        const value = structuredClone(snapshot)
        Reflect.set(Reflect.get(value, 'room') as object, 'rulesVersion', 8)
        return value
      })()],
      ['lair cap', (() => {
        const value = structuredClone(snapshot)
        const room = Reflect.get(value, 'room') as { configuration: { environment: { lairs: object } } }
        Reflect.set(room.configuration.environment.lairs, 'maxLivingMobs', 2)
        return value
      })()],
      ['timer type', (() => {
        const value = structuredClone(lairSnapshot)
        const state = Reflect.get(value, 'state') as { lairs: object[] }
        Reflect.set(state.lairs[0]!, 'spawnRemainingTicks', 'now')
        return value
      })()],
      ['mob total', (() => {
        const value = structuredClone(snapshot)
        const state = Reflect.get(value, 'state') as { mobs: unknown[] }
        state.mobs = [{}, {}, {}]
        return value
      })()],
    ]
    for (const [label, value] of cases) {
      expect(original(value), `original accepted ${label}`).toBe(false)
      expect(standalone(value), `standalone accepted ${label}`).toBe(false)
      expect(errors(standalone)).toEqual(errors(original))
    }
  })

  it('retains error objects across invalid-valid-invalid transitions', () => {
    const invalid = { type: 'authenticated', protocolVersion: 16 }
    expectParity(invalid, false)
    const retainedGenerated = standalone.errors
    const retainedOriginal = original.errors
    expectParity({ type: 'authenticated', protocolVersion: 17 }, true)
    expect(retainedGenerated).not.toBeNull()
    expect(retainedOriginal).not.toBeNull()
    expectParity(invalid, false)
    expect(standalone.errors).not.toBe(retainedGenerated)
    expect(original.errors).not.toBe(retainedOriginal)
  })
})
