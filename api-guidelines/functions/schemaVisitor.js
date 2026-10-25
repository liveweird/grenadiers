const CHILD_SCHEMA_KEYS = ['additionalProperties', 'items', 'not']
const CHILD_SCHEMA_ARRAY_KEYS = ['allOf', 'anyOf', 'oneOf']

export function visitSchemas(document, visitor) {
  const visited = new WeakSet()

  function visitSchema(schema, path) {
    if (schema === null || typeof schema !== 'object' || Array.isArray(schema)) return
    if (visited.has(schema)) return
    visited.add(schema)
    visitor(schema, path)

    for (const [name, child] of Object.entries(schema.properties ?? {})) {
      visitSchema(child, [...path, 'properties', name])
    }
    for (const key of CHILD_SCHEMA_KEYS) visitSchema(schema[key], [...path, key])
    for (const key of CHILD_SCHEMA_ARRAY_KEYS) {
      if (!Array.isArray(schema[key])) continue
      schema[key].forEach((child, index) => visitSchema(child, [...path, key, index]))
    }
  }

  for (const [name, schema] of Object.entries(document?.components?.schemas ?? {})) {
    visitSchema(schema, ['components', 'schemas', name])
  }

  function findInlineSchemas(value, path) {
    if (Array.isArray(value)) {
      value.forEach((item, index) => findInlineSchemas(item, [...path, index]))
      return
    }
    if (value === null || typeof value !== 'object') return

    for (const [key, child] of Object.entries(value)) {
      if (key === 'example' || key === 'examples') continue
      if (key === 'schema') visitSchema(child, [...path, key])
      else findInlineSchemas(child, [...path, key])
    }
  }

  findInlineSchemas(document, [])
}
