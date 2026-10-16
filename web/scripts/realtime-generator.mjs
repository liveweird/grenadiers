function refName(ref) {
  return ref.slice(ref.lastIndexOf('/') + 1)
}

function typeFor(schema) {
  if ('$ref' in schema) return refName(schema.$ref)
  if ('const' in schema) return JSON.stringify(schema.const)
  if (schema.oneOf) return schema.oneOf.map(typeFor).join(' | ')
  if (schema.enum) return schema.enum.map((value) => JSON.stringify(value)).join(' | ')
  if (schema.type === 'integer' || schema.type === 'number') return 'number'
  if (schema.type === 'boolean') return 'boolean'
  if (schema.type === 'array') return `${typeFor(schema.items)}[]`
  if (schema.type === 'object') return 'Record<string, unknown>'
  return 'string'
}

function definition(name, schema) {
  if (schema.oneOf) return `export type ${name} = ${schema.oneOf.map(typeFor).join(' | ')}\n`
  if (schema.type !== 'object') return `export type ${name} = ${typeFor(schema)}\n`
  const required = new Set(schema.required ?? [])
  const properties = Object.entries(schema.properties ?? {}).map(
    ([property, value]) => `  ${JSON.stringify(property)}${required.has(property) ? '' : '?'}: ${typeFor(value)};`,
  )
  return `export interface ${name} {\n${properties.join('\n')}\n}\n`
}

export function generateRealtime(schema) {
  const definitions = Object.entries(schema.$defs ?? {}).map(([name, value]) => definition(name, value))
  return `// This file was auto-generated from api/realtime.schema.json. Do not edit.\n\n${definitions.join('\n')}`
}
