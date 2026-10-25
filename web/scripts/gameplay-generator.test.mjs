import { readFile } from 'node:fs/promises'
import ts from 'typescript'
import { expect, test } from 'vitest'

import { discoverGameplayContracts } from './gameplay-contract-inventory.mjs'
import { generateGameplay } from './gameplay-generator.mjs'
import { generateRealtime } from './realtime-generator.mjs'

test('all gameplay contracts remain byte-for-byte reproducible', async () => {
  for (const { version, spec, target, sourceLabel } of await discoverGameplayContracts()) {
    const schema = JSON.parse(await readFile(spec, 'utf8'))
    const generated = generateGameplay(schema, sourceLabel, version)
    expect(generated).toBe(await readFile(target, 'utf8'))
  }
})

test('gameplay v2-v6 generation stays byte-for-byte identical to the generic type generator', async () => {
  const contracts = await discoverGameplayContracts()
  for (const { version, spec, sourceLabel } of contracts.filter(({ version }) => version <= 6)) {
    const schema = JSON.parse(await readFile(spec, 'utf8'))
    expect(generateGameplay(schema, sourceLabel, version)).toBe(generateRealtime(schema, sourceLabel))
  }
})

function syntheticSchema(itemIds, pickupKinds) {
  return {
    $defs: {
      GameplayVersion: { const: 7 },
      PickupKind: { enum: pickupKinds },
      ItemId: { enum: itemIds },
    },
  }
}

test.each([
  [['shield', 'heart'], ['bombCapacity', 'heart']],
  [['teleporter', 'kick', 'oilBarrel'], ['movementSpeed', 'oilBarrel']],
])('emits each contract ItemId membership and order independently of PickupKind', (items, pickups) => {
  const generated = generateGameplay(syntheticSchema(items, pickups), 'synthetic.json', 7)
  expect(generated).toContain(`export const ITEM_IDS = ${JSON.stringify(items, null, 2)} as const`)
  expect(generated).not.toContain(`export const ITEM_IDS = ${JSON.stringify(pickups, null, 2)} as const`)
})

test.each([
  ['missing', undefined, /expected an enum/],
  ['empty', [], /at least one item ID/],
  ['duplicate', ['heart', 'heart'], /must be unique/],
  ['non-string', ['heart', 7], /must be strings/],
])('rejects a %s ItemId enum', (_label, itemIds, expected) => {
  const schema = syntheticSchema(['placeholder'], ['different'])
  if (itemIds === undefined) delete schema.$defs.ItemId
  else schema.$defs.ItemId.enum = itemIds
  expect(() => generateGameplay(schema, 'synthetic.json', 7)).toThrow(expected)
})

test('generated catalog compiles to a literal value with no runtime imports', async () => {
  const items = ['wallPass', 'heart']
  const generated = generateGameplay(syntheticSchema(items, ['bombCapacity']), 'synthetic.json', 7)
  const compiled = ts.transpileModule(generated, {
    compilerOptions: { module: ts.ModuleKind.ESNext, target: ts.ScriptTarget.ES2022 },
  }).outputText
  expect(compiled.split('\n').some((line) => line.trimStart().startsWith('import '))).toBe(false)
  expect(compiled).not.toContain('schema.json')
  const module = await import(`data:text/javascript,${encodeURIComponent(compiled)}`)
  expect(module.ITEM_IDS).toEqual(items)
})
