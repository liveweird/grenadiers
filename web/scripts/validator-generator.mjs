import { readFile } from 'node:fs/promises'
import { join } from 'node:path'
import { fileURLToPath, pathToFileURL } from 'node:url'
import Ajv from 'ajv'
import standaloneCode from 'ajv/dist/standalone/index.js'
import ts from 'typescript'
import { build } from 'vite'

const validatorDefinitions = {
  'gameplay-v11': {
    schema: '../api/gameplay-v11.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v11/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  realtime: {
    schema: '../api/realtime.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { ServerMessage } from '../../realtime/schema'

declare const validate: {
  (value: unknown): value is ServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile: (ajv, schema) => ajv.compile(schema),
  },
}

const validatorNames = Object.freeze(Object.keys(validatorDefinitions))

function directoryUrl(value) {
  if (typeof value === 'string') return pathToFileURL(`${value}/`)
  if (value?.protocol === 'file:') return new URL(value.href)
  throw new TypeError('validator directories must be filesystem paths or file URLs')
}

function assertSingleSelfContainedChunk(output, name) {
  const chunks = output.filter((item) => item.type === 'chunk')
  if (chunks.length !== 1) throw new Error(`${name} validator build emitted ${chunks.length} chunks`)
  const [chunk] = chunks
  if (chunk.imports.length > 0 || chunk.dynamicImports.length > 0) {
    throw new Error(`${name} validator build is not self-contained`)
  }
  assertNoExecutableCodeGeneration(chunk.code, name)
  return chunk.code
}

function propertyName(expression) {
  if (ts.isPropertyAccessExpression(expression)) return expression.name.text
  if (ts.isElementAccessExpression(expression) && expression.argumentExpression
    && (ts.isStringLiteral(expression.argumentExpression)
      || ts.isNoSubstitutionTemplateLiteral(expression.argumentExpression))) {
    return expression.argumentExpression.text
  }
  return null
}

function isComparison(operator) {
  return [ts.SyntaxKind.EqualsEqualsToken, ts.SyntaxKind.EqualsEqualsEqualsToken,
    ts.SyntaxKind.ExclamationEqualsToken, ts.SyntaxKind.ExclamationEqualsEqualsToken].includes(operator)
}

function isBenignConstructorComparison(node) {
  let expression = node
  while (expression.parent && (ts.isParenthesizedExpression(expression.parent)
    || ts.isAsExpression(expression.parent) || ts.isNonNullExpression(expression.parent)
    || ts.isTypeAssertionExpression(expression.parent))) {
    expression = expression.parent
  }
  return Boolean(expression.parent && ts.isBinaryExpression(expression.parent)
    && isComparison(expression.parent.operatorToken.kind))
}

export function assertNoExecutableCodeGeneration(code, name = 'standalone') {
  const source = ts.createSourceFile(`${name}.mjs`, code, ts.ScriptTarget.Latest, true, ts.ScriptKind.JS)
  if (source.parseDiagnostics.length > 0) {
    throw new Error(`${name} validator is not valid JavaScript: ${source.parseDiagnostics[0].messageText}`)
  }
  function reject(node, forbidden) {
    const position = source.getLineAndCharacterOfPosition(node.getStart(source))
    throw new Error(`${name} validator contains executable ${forbidden} at ${position.line + 1}:${position.character + 1}`)
  }
  function visit(node) {
    if (ts.isCallExpression(node) && node.expression.kind === ts.SyntaxKind.ImportKeyword) {
      reject(node, 'import()')
    }
    if (ts.isIdentifier(node) && ['require', 'eval', 'Function'].includes(node.text)) {
      reject(node, node.text)
    }
    if (ts.isPropertyAccessExpression(node) || ts.isElementAccessExpression(node)) {
      const property = propertyName(node)
      if (property && ['require', 'eval', 'Function'].includes(property)) reject(node, property)
      if (property === 'constructor' && !isBenignConstructorComparison(node)) reject(node, 'constructor')
    }
    ts.forEachChild(node, visit)
  }
  visit(source)
}

export async function bundleStandaloneValidator(name, commonJs, webDirectory) {
  const webUrl = directoryUrl(webDirectory)
  const entry = `/grenadiers-validator-entry/${name}.cjs`
  const entryUrl = pathToFileURL(entry).href
  const result = await build({
    root: fileURLToPath(webUrl),
    configFile: false,
    logLevel: 'silent',
    plugins: [{
      name: 'grenadiers-standalone-validator',
      resolveId(id) { if (id === entry || id === entryUrl) return entry },
      load(id) { if (id === entry) return commonJs },
    }],
    build: {
      write: false,
      minify: false,
      sourcemap: false,
      lib: { entry, formats: ['es'], fileName: name },
    },
  })
  const outputs = Array.isArray(result) ? result.flatMap((item) => item.output) : result.output
  return {
    code: assertSingleSelfContainedChunk(outputs, name),
  }
}

async function generateValidator(name, schema, webDirectory) {
  const definition = validatorDefinitions[name]
  if (!definition) throw new Error(`Unknown validator ${name}`)
  const ajv = new Ajv({ allErrors: true, code: { source: true } })
  const commonJs = standaloneCode(ajv, definition.compile(ajv, schema))
  const bundled = await bundleStandaloneValidator(name, commonJs, webDirectory)
  return { ...bundled, declaration: definition.declaration }
}

export async function generateValidators(webDirectory = join(import.meta.dirname, '..')) {
  const webUrl = directoryUrl(webDirectory)
  const generated = new Map()
  for (const name of validatorNames) {
    const definition = validatorDefinitions[name]
    const schema = JSON.parse(await readFile(new URL(definition.schema, webUrl), 'utf8'))
    generated.set(name, await generateValidator(name, schema, webUrl))
  }
  return generated
}

export async function findValidatorDrift(
  webDirectory = join(import.meta.dirname, '..'),
  targetDirectory = join(import.meta.dirname, '..', 'src', 'validation', 'generated'),
) {
  const targetUrl = directoryUrl(targetDirectory)
  const stale = []
  for (const [name, generated] of await generateValidators(webDirectory)) {
    for (const [extension, expected] of [['mjs', generated.code], ['d.mts', generated.declaration]]) {
      const actual = await readFile(new URL(`${name}.${extension}`, targetUrl), 'utf8')
      if (actual !== expected) stale.push(`${name}.${extension}`)
    }
  }
  return stale
}
