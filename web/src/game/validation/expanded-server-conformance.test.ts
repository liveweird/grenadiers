import { describe, expect, it } from 'vitest'
import fixtures from '../../../../api/fixtures/gameplay-expanded-server-conformance.json'
import { validateServerMessage } from '../client/clientProtocol'
import { validateGameplayV10ServerMessage } from '../v10/validation'
import { validateGameplayV11ServerMessage } from '../v11/validation'
import { validateGameplayV8ServerMessage } from '../v8/validation'
import { validateGameplayV9ServerMessage } from '../v9/validation'

type RecordValue = Record<string, unknown>
type Result = { valid: boolean; source?: 'schema' | 'semantics'; error?: string }
type Validator = (value: unknown) => Result

const validators = new Map<number, Validator>([
  [8, validateGameplayV8ServerMessage],
  [9, validateGameplayV9ServerMessage],
  [10, validateGameplayV10ServerMessage],
  [11, validateGameplayV11ServerMessage],
])

function record(value: unknown): RecordValue {
  return value as RecordValue
}

function frame(caseId: string): RecordValue {
  const fixture = fixtures.cases.find((candidate) => candidate.caseId === caseId)
  if (!fixture) throw new Error(`missing fixture ${caseId}`)
  return JSON.parse(fixture.frame) as RecordValue
}

describe('expanded gameplay server conformance', () => {
  it('accepts actual Rust-produced frames through each retained validator', () => {
    for (const fixture of fixtures.cases) {
      const validate = validators.get(fixture.version)
      if (!validate) throw new Error(`missing validator for v${fixture.version}`)
      expect(validate(JSON.parse(fixture.frame)), fixture.caseId).toEqual(expect.objectContaining({
        valid: true,
      }))
      if (fixture.version === 11) {
        const recipient = fixture.recipient === 'one' || fixture.recipient === 'two'
          ? fixture.recipient : null
        expect(validateServerMessage(JSON.parse(fixture.frame), recipient), fixture.caseId).toBe(true)
      }
    }
  })

  it('covers the required version transitions and recipient pairs', () => {
    for (const version of [8, 9, 10, 11]) {
      const cases = fixtures.cases.filter((fixture) => fixture.version === version)
      expect(cases.some((fixture) => fixture.scenario === 'waitingAdmission')).toBe(true)
      const runningRecipients = cases
        .filter((fixture) => fixture.scenario.startsWith('running'))
        .map((fixture) => fixture.recipient)
      expect(new Set(runningRecipients)).toEqual(new Set(['one', 'two']))
    }
    const frozenActors = record(frame('v9-runningFrozen-one').state).actors as RecordValue[]
    expect(frozenActors.map((actor) => actor.voluntaryActionsFrozen)).toEqual([false, true])
    const nullActors = record(frame('v10-runningTeleportNull-one').state).actors as RecordValue[]
    const movedActors = record(frame('v10-runningTeleported-two').state).actors as RecordValue[]
    expect(nullActors.map((actor) => actor.lastTeleportTick)).toEqual([null, null])
    expect(movedActors.map((actor) => actor.lastTeleportTick)).toEqual([181, 181])
    expect(record(frame('v11-runningOilEmpty-one').state).oilBarrels).toEqual([])
    const oilFrame = frame('v11-runningOilMaterialized-one')
    expect(record(oilFrame.state).oilBarrels).toHaveLength(2)
    expect(JSON.stringify(oilFrame)).not.toContain('concealed')
  })

  it('pins archived recipient privacy for v8 and v10 projection pairs', () => {
    for (const [oneId, twoId] of [
      ['v8-runningPrivacy-one', 'v8-runningPrivacy-two'],
      ['v10-runningTeleported-one', 'v10-runningTeleported-two'],
    ] as const) {
      const oneActors = record(frame(oneId).state).actors as RecordValue[]
      const twoActors = record(frame(twoId).state).actors as RecordValue[]
      expect(oneActors[1]).toEqual({ type: 'hidden', id: 2, seat: 'two', alive: true })
      expect(twoActors[1]).toMatchObject({
        type: 'visible', id: 2, seat: 'two',
        effects: [{ kind: 'invisibilityCloak', remainingTicks: 600 }],
      })
    }
  })

  it('keeps structural and semantic rejection sources distinct', () => {
    const structural = structuredClone(frame('v11-runningOilMaterialized-two'))
    delete structural.room
    expect(validateGameplayV11ServerMessage(structural)).toMatchObject({
      valid: false, source: 'schema',
    })

    const semantic = structuredClone(frame('v11-runningOilMaterialized-two'))
    const settings = record(record(semantic.room).settings)
    settings.enabledItems = (settings.enabledItems as string[]).filter((item) => item !== 'oilBarrel')
    expect(validateGameplayV11ServerMessage(semantic)).toEqual({
      valid: false, source: 'semantics', error: 'invalidOilBarrels',
    })
  })

  it('exercises live v11 recipient privacy and identity checks after generic validation', () => {
    const forOne = frame('v11-runningOilMaterialized-one')
    const forTwo = frame('v11-runningOilMaterialized-two')
    expect(validateGameplayV11ServerMessage(forOne).valid).toBe(true)
    expect(validateServerMessage(forOne, 'one')).toBe(true)
    expect(validateServerMessage(forOne, 'two')).toBe(false)
    const oneActors = record(forOne.state).actors as RecordValue[]
    const twoActors = record(forTwo.state).actors as RecordValue[]
    expect(oneActors[1]?.type).toBe('hidden')
    expect(twoActors[1]?.type).toBe('visible')

    const duplicateIdentity = structuredClone(forTwo)
    const actors = record(duplicateIdentity.state).actors as RecordValue[]
    actors[1]!.id = actors[0]!.id
    expect(validateGameplayV11ServerMessage(duplicateIdentity).valid).toBe(true)
    expect(validateServerMessage(duplicateIdentity, 'two')).toBe(false)
  })

  it('rejects changed physical state in the preceding retained version', () => {
    for (const [caseId, validate] of [
      ['v9-runningFrozen-one', validateGameplayV8ServerMessage],
      ['v10-runningTeleported-one', validateGameplayV9ServerMessage],
      ['v11-runningOilMaterialized-one', validateGameplayV10ServerMessage],
    ] as const) {
      expect(validate(frame(caseId)), caseId).toMatchObject({ valid: false, source: 'schema' })
    }
  })
})
