import { generateRealtime } from './realtime-generator.mjs'

const firstItemCatalogVersion = 7

function itemIdsFrom(schema) {
  const itemId = schema?.$defs?.ItemId
  if (itemId === null || typeof itemId !== 'object' || Array.isArray(itemId)
    || !Array.isArray(itemId.enum)) {
    throw new Error('Unsupported gameplay schema at $defs["ItemId"]: expected an enum')
  }
  if (itemId.enum.length === 0) {
    throw new Error('Unsupported gameplay schema at $defs["ItemId"].enum: expected at least one item ID')
  }
  if (itemId.enum.some((item) => typeof item !== 'string')) {
    throw new Error('Unsupported gameplay schema at $defs["ItemId"].enum: item IDs must be strings')
  }
  if (new Set(itemId.enum).size !== itemId.enum.length) {
    throw new Error('Unsupported gameplay schema at $defs["ItemId"].enum: item IDs must be unique')
  }
  return itemId.enum
}

export function generateGameplay(schema, sourceLabel, version) {
  if (version < firstItemCatalogVersion) return generateRealtime(schema, sourceLabel)

  const itemIds = itemIdsFrom(schema)
  const generated = generateRealtime(schema, sourceLabel)
  const literal = JSON.stringify(itemIds, null, 2)
  return `${generated}\nexport const ITEM_IDS = ${literal} as const satisfies readonly ItemId[]\n`
}
