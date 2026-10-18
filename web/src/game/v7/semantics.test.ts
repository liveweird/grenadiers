import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import fixtures from '../../../../api/fixtures/gameplay-v7-semantics.json'
import gameplayV7Schema from '../../../../api/gameplay-v7.schema.json'
import { effectSemanticError, itemSetSemanticError } from './semantics'
import type { EffectView, ItemId } from './schema'

const ajv = new Ajv().addSchema(gameplayV7Schema)
const validateEffects = ajv.compile({
  type: 'array',
  maxItems: 16,
  items: { $ref: `${gameplayV7Schema.$id}#/$defs/EffectView` },
})
const validateItems = ajv.compile({
  type: 'array',
  maxItems: 22,
  uniqueItems: true,
  items: { $ref: `${gameplayV7Schema.$id}#/$defs/ItemId` },
})

describe('gameplay v7 semantic fixtures', () => {
  it('enforces ordinary effect uniqueness and the independent slow-foot cap', () => {
    for (const fixture of fixtures.effects) {
      expect(validateEffects(fixture.value), `${fixture.name} must be structurally valid`).toBe(true)
      expect(effectSemanticError(fixture.value as EffectView[]), fixture.name).toBe(fixture.expectedError)
    }
  })

  it('requires enabled items to be available in the containing room', () => {
    for (const fixture of fixtures.itemSets) {
      expect(validateItems(fixture.enabledItems), `${fixture.name}.enabledItems`).toBe(true)
      expect(validateItems(fixture.availableItems), `${fixture.name}.availableItems`).toBe(true)
      expect(
        itemSetSemanticError(fixture.enabledItems as ItemId[], fixture.availableItems as ItemId[]),
        fixture.name,
      ).toBe(fixture.expectedError)
    }
  })
})
