const annotationKeywords = new Set(['description', 'title'])
const supportedKeywords = new Set([
  ...annotationKeywords,
  '$ref',
  'additionalProperties',
  'const',
  'enum',
  'items',
  'maxItems',
  'maxLength',
  'maximum',
  'minItems',
  'minLength',
  'minimum',
  'oneOf',
  'pattern',
  'properties',
  'required',
  'type',
  'uniqueItems',
])
const reservedTypeNames = new Set([
  'Record',
  'abstract',
  'any',
  'as',
  'asserts',
  'async',
  'await',
  'bigint',
  'boolean',
  'break',
  'case',
  'catch',
  'class',
  'const',
  'constructor',
  'continue',
  'debugger',
  'declare',
  'default',
  'delete',
  'do',
  'else',
  'enum',
  'export',
  'extends',
  'false',
  'finally',
  'for',
  'from',
  'function',
  'get',
  'global',
  'if',
  'implements',
  'import',
  'in',
  'infer',
  'instanceof',
  'interface',
  'intrinsic',
  'is',
  'keyof',
  'let',
  'module',
  'namespace',
  'never',
  'new',
  'null',
  'number',
  'object',
  'of',
  'out',
  'override',
  'package',
  'private',
  'protected',
  'public',
  'readonly',
  'require',
  'return',
  'satisfies',
  'set',
  'static',
  'string',
  'super',
  'switch',
  'symbol',
  'this',
  'throw',
  'true',
  'try',
  'type',
  'typeof',
  'undefined',
  'unique',
  'unknown',
  'using',
  'var',
  'void',
  'while',
  'with',
  'yield',
])

function fail(path, message) {
  throw new Error(`Unsupported realtime schema at ${path}: ${message}`)
}

function assertRecord(value, path) {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) {
    fail(path, 'expected a schema object')
  }
}

function assertKnownKeywords(schema, path) {
  for (const keyword of Object.keys(schema)) {
    if (!supportedKeywords.has(keyword)) fail(path, `keyword ${JSON.stringify(keyword)}`)
  }
}

function literalType(value, path) {
  if (typeof value === 'number' && !Number.isFinite(value)) {
    fail(path, 'numeric literals must be finite')
  }
  if (value === null || ['string', 'number', 'boolean'].includes(typeof value)) {
    return JSON.stringify(value)
  }
  fail(path, 'const and enum values must be string, number, boolean, or null literals')
}

function refName(ref, definitions, path) {
  if (typeof ref !== 'string') fail(path, '$ref must be a string')
  const match = /^#\/\$defs\/([^/]+)$/.exec(ref)
  if (!match) fail(path, `$ref ${JSON.stringify(ref)} must point directly into #/$defs`)

  const name = match[1].replaceAll('~1', '/').replaceAll('~0', '~')
  if (!Object.hasOwn(definitions, name)) fail(path, `$ref ${JSON.stringify(ref)} does not exist`)
  if (!/^[A-Za-z_$][\w$]*$/.test(name)) {
    fail(path, `$ref target ${JSON.stringify(name)} is not a TypeScript identifier`)
  }
  return name
}

function validateDefinitionName(name, path) {
  if (!/^[A-Za-z_$][\w$]*$/.test(name)) fail(path, 'definition name is not a TypeScript identifier')
  if (reservedTypeNames.has(name)) fail(path, `definition name ${JSON.stringify(name)} is reserved by TypeScript`)
}

function structuralKeywordCount(schema) {
  return ['$ref', 'const', 'oneOf', 'enum', 'type'].filter((keyword) => keyword in schema).length
}

function assertOnlyKeywords(schema, allowed, path) {
  for (const keyword of Object.keys(schema)) {
    if (!allowed.has(keyword) && !annotationKeywords.has(keyword)) {
      fail(path, `keyword ${JSON.stringify(keyword)} is not supported for this schema shape`)
    }
  }
}

function validateBoundedCount(schema, minimumKeyword, maximumKeyword, path) {
  for (const keyword of [minimumKeyword, maximumKeyword]) {
    if (keyword in schema && (!Number.isSafeInteger(schema[keyword]) || schema[keyword] < 0)) {
      fail(path, `${keyword} must be a non-negative safe integer`)
    }
  }
  if (
    minimumKeyword in schema &&
    maximumKeyword in schema &&
    schema[minimumKeyword] > schema[maximumKeyword]
  ) {
    fail(path, `${minimumKeyword} must not exceed ${maximumKeyword}`)
  }
}

function validateSchema(schema, definitions, path) {
  assertRecord(schema, path)
  assertKnownKeywords(schema, path)

  if (structuralKeywordCount(schema) !== 1) {
    fail(path, 'expected exactly one of $ref, const, oneOf, enum, or type')
  }

  if ('$ref' in schema) {
    assertOnlyKeywords(schema, new Set(['$ref']), path)
    refName(schema.$ref, definitions, path)
    return
  }
  if ('const' in schema) {
    assertOnlyKeywords(schema, new Set(['const']), path)
    literalType(schema.const, path)
    return
  }
  if ('enum' in schema) {
    assertOnlyKeywords(schema, new Set(['enum']), path)
    if (!Array.isArray(schema.enum) || schema.enum.length === 0) fail(path, 'enum must be non-empty')
    schema.enum.forEach((value) => literalType(value, path))
    return
  }
  if ('oneOf' in schema) {
    assertOnlyKeywords(schema, new Set(['oneOf']), path)
    if (!Array.isArray(schema.oneOf) || schema.oneOf.length === 0) fail(path, 'oneOf must be non-empty')
    schema.oneOf.forEach((member, index) => validateSchema(member, definitions, `${path}.oneOf[${index}]`))
    return
  }

  if (!['array', 'boolean', 'integer', 'null', 'number', 'object', 'string'].includes(schema.type)) {
    fail(path, `type ${JSON.stringify(schema.type)}`)
  }
  if (schema.type === 'array') {
    assertOnlyKeywords(schema, new Set(['type', 'items', 'minItems', 'maxItems', 'uniqueItems']), path)
    if (!('items' in schema)) fail(path, 'arrays require items')
    validateBoundedCount(schema, 'minItems', 'maxItems', path)
    if ('uniqueItems' in schema && typeof schema.uniqueItems !== 'boolean') {
      fail(path, 'uniqueItems must be a boolean')
    }
    validateSchema(schema.items, definitions, `${path}.items`)
  } else if (schema.type === 'object') {
    assertOnlyKeywords(schema, new Set(['type', 'additionalProperties', 'properties', 'required']), path)
    validateObject(schema, definitions, path)
  } else {
    validatePrimitive(schema, path)
  }
}

function validateObject(schema, definitions, path) {
  if (schema.additionalProperties !== false) {
    fail(path, 'only closed objects with additionalProperties: false are supported')
  }
  const properties = schema.properties ?? {}
  assertRecord(properties, `${path}.properties`)
  for (const [name, property] of Object.entries(properties)) {
    validateSchema(property, definitions, `${path}.properties[${JSON.stringify(name)}]`)
  }

  const required = schema.required ?? []
  if (!Array.isArray(required) || required.some((name) => typeof name !== 'string')) {
    fail(path, 'required must be an array of property names')
  }
  if (new Set(required).size !== required.length) fail(path, 'required contains duplicates')
  for (const name of required) {
    if (!Object.hasOwn(properties, name)) fail(path, `required property ${JSON.stringify(name)} is not defined`)
  }
}

function unguardedReferences(schema, definitions, path) {
  if ('$ref' in schema) return [refName(schema.$ref, definitions, path)]
  if (schema.oneOf) {
    return schema.oneOf.flatMap((member, index) =>
      unguardedReferences(member, definitions, `${path}.oneOf[${index}]`),
    )
  }
  return []
}

function rejectUnguardedAliasCycles(definitions) {
  const graph = new Map(
    Object.entries(definitions).map(([name, schema]) => [
      name,
      schema.type === 'object' ? [] : unguardedReferences(schema, definitions, `$defs[${JSON.stringify(name)}]`),
    ]),
  )
  const visiting = new Set()
  const visited = new Set()

  function visit(name, trail) {
    if (visiting.has(name)) fail(`$defs[${JSON.stringify(name)}]`, `unguarded alias cycle: ${[...trail, name].join(' -> ')}`)
    if (visited.has(name)) return
    visiting.add(name)
    for (const referenced of graph.get(name) ?? []) visit(referenced, [...trail, name])
    visiting.delete(name)
    visited.add(name)
  }

  for (const name of graph.keys()) visit(name, [])
}

function validatePrimitive(schema, path) {
  let allowed = new Set(['type'])
  if (['integer', 'number'].includes(schema.type)) allowed = new Set(['type', 'minimum', 'maximum'])
  if (schema.type === 'string') allowed = new Set(['type', 'minLength', 'maxLength', 'pattern'])
  assertOnlyKeywords(schema, allowed, path)
  if (schema.type === 'string') {
    validateBoundedCount(schema, 'minLength', 'maxLength', path)
    if ('pattern' in schema) {
      if (typeof schema.pattern !== 'string') fail(path, 'pattern must be a string')
      try {
        new RegExp(schema.pattern, 'u')
      } catch {
        fail(path, 'pattern must be a valid ECMAScript Unicode regular expression')
      }
    }
  }
  for (const keyword of ['minimum', 'maximum']) {
    if (keyword in schema && typeof schema[keyword] !== 'number') fail(path, `${keyword} must be a number`)
  }
}

function typeFor(schema, definitions, path, indent = 0) {
  if ('$ref' in schema) return refName(schema.$ref, definitions, path)
  if ('const' in schema) return literalType(schema.const, path)
  if (schema.oneOf) {
    return schema.oneOf.map((value, index) => typeFor(value, definitions, `${path}.oneOf[${index}]`, indent)).join(' | ')
  }
  if (schema.enum) return schema.enum.map((value) => literalType(value, path)).join(' | ')
  if (schema.type === 'integer' || schema.type === 'number') return 'number'
  if (schema.type === 'boolean') return 'boolean'
  if (schema.type === 'string') return 'string'
  if (schema.type === 'null') return 'null'
  if (schema.type === 'array') {
    const item = typeFor(schema.items, definitions, `${path}.items`, indent)
    const itemIsUnion = schema.items.oneOf || (schema.items.enum && schema.items.enum.length > 1)
    const itemType = itemIsUnion ? `(${item})` : item
    return `${itemType}[]`
  }
  if (schema.type === 'object') return objectBody(schema, definitions, path, indent)
  fail(path, 'schema was not validated')
}

function objectProperties(schema, definitions, path, indent) {
  const required = new Set(schema.required ?? [])
  const padding = '  '.repeat(indent)
  return Object.entries(schema.properties ?? {}).map(([property, value]) => {
    const propertyType = typeFor(value, definitions, `${path}.properties[${JSON.stringify(property)}]`, indent)
    return `${padding}${JSON.stringify(property)}${required.has(property) ? '' : '?'}: ${propertyType};`
  })
}

function objectBody(schema, definitions, path, indent) {
  const properties = objectProperties(schema, definitions, path, indent + 1)
  if (properties.length === 0) return 'Record<string, never>'
  const closingPadding = '  '.repeat(indent)
  return `{\n${properties.join('\n')}\n${closingPadding}}`
}

function definition(name, schema, definitions) {
  const path = `$defs[${JSON.stringify(name)}]`
  validateDefinitionName(name, path)
  if (schema.type === 'object' && Object.keys(schema.properties ?? {}).length === 0) {
    return `export type ${name} = Record<string, never>\n`
  }
  if (schema.type !== 'object') return `export type ${name} = ${typeFor(schema, definitions, path)}\n`
  const properties = objectProperties(schema, definitions, path, 1)
  return `export interface ${name} {\n${properties.join('\n')}\n}\n`
}

function validateDocument(schema) {
  assertRecord(schema, '$')
  const rootKeywords = new Set(['$schema', '$id', 'title', 'description', '$ref', '$defs'])
  for (const keyword of Object.keys(schema)) {
    if (!rootKeywords.has(keyword)) fail('$', `keyword ${JSON.stringify(keyword)}`)
  }
  const definitions = schema.$defs ?? {}
  assertRecord(definitions, '$.$defs')
  for (const name of Object.keys(definitions)) validateDefinitionName(name, `$defs[${JSON.stringify(name)}]`)
  for (const [name, value] of Object.entries(definitions)) {
    validateSchema(value, definitions, `$defs[${JSON.stringify(name)}]`)
  }
  if ('$ref' in schema) refName(schema.$ref, definitions, '$')
  rejectUnguardedAliasCycles(definitions)
  return definitions
}

export function generateRealtime(schema) {
  const definitions = validateDocument(schema)
  const generatedDefinitions = Object.entries(definitions).map(([name, value]) => definition(name, value, definitions))
  return `// This file was auto-generated from api/realtime.schema.json. Do not edit.\n\n${generatedDefinitions.join('\n')}`
}
