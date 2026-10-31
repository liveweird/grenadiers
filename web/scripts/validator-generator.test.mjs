import assert from 'node:assert/strict'
import { execFileSync } from 'node:child_process'
import { cp, mkdtemp, mkdir, readFile, rm, symlink, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { pathToFileURL } from 'node:url'
import { afterEach, test } from 'vitest'
import { assertNoExecutableCodeGeneration, bundleStandaloneValidator,
  generateValidators } from './validator-generator.mjs'

const temporaryDirectories = []
afterEach(async () => {
  await Promise.all(temporaryDirectories.splice(0).map((directory) => rm(directory, { recursive: true, force: true })))
})

async function temporaryCheckout(nested = false) {
  const temporaryRoot = await mkdtemp(join(tmpdir(), 'grenadiers-validators-'))
  temporaryDirectories.push(temporaryRoot)
  const checkout = nested ? join(temporaryRoot, 'different', 'nested', 'checkout') : temporaryRoot
  await mkdir(join(checkout, 'web', 'src', 'validation', 'generated'), { recursive: true })
  await mkdir(join(checkout, 'web', 'scripts'))
  await mkdir(join(checkout, 'api'))
  await Promise.all([
    cp(join(import.meta.dirname, '..', '..', 'api', 'cpu-debug.schema.json'), join(checkout, 'api', 'cpu-debug.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v11.schema.json'), join(checkout, 'api', 'gameplay-v11.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v12.schema.json'), join(checkout, 'api', 'gameplay-v12.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v13.schema.json'), join(checkout, 'api', 'gameplay-v13.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v14.schema.json'), join(checkout, 'api', 'gameplay-v14.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v15.schema.json'), join(checkout, 'api', 'gameplay-v15.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v16.schema.json'), join(checkout, 'api', 'gameplay-v16.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'gameplay-v17.schema.json'), join(checkout, 'api', 'gameplay-v17.schema.json')),
    cp(join(import.meta.dirname, '..', '..', 'api', 'realtime.schema.json'), join(checkout, 'api', 'realtime.schema.json')),
    cp(join(import.meta.dirname, 'validator-generator.mjs'), join(checkout, 'web', 'scripts', 'validator-generator.mjs')),
  ])
  const sourceModules = join(import.meta.dirname, '..', 'node_modules')
  const targetModules = join(checkout, 'web', 'node_modules')
  await mkdir(targetModules)
  await Promise.all(['ajv', 'fast-deep-equal', 'fast-uri', 'json-schema-traverse', 'require-from-string']
    .map((name) => cp(join(sourceModules, name), join(targetModules, name), { recursive: true })))
  await Promise.all(['typescript', 'vite']
    .map((name) => symlink(join(sourceModules, name), join(targetModules, name), 'dir')))
  return checkout
}

function standaloneCallWithModule(cwd, modulePath, expression, ...arguments_) {
  const moduleUrl = pathToFileURL(modulePath).href
  const source = `import * as generator from ${JSON.stringify(moduleUrl)};
const result = await (${expression})(generator, ...JSON.parse(process.argv[1]));
process.stdout.write(JSON.stringify(result));`
  const output = execFileSync(process.execPath, ['--input-type=module', '--eval', source, JSON.stringify(arguments_)], {
    cwd, encoding: 'utf8', maxBuffer: 4 * 1024 * 1024,
  })
  return JSON.parse(output)
}

function standaloneCallIn(cwd, expression, ...arguments_) {
  return standaloneCallWithModule(cwd, join(cwd, 'scripts', 'validator-generator.mjs'), expression, ...arguments_)
}

test('checked-in validators and declarations are reproducible and executable', async () => {
  const generated = await generateValidators()
  for (const [name, artifacts] of generated) {
    assert.equal(artifacts.code, await readFile(join(import.meta.dirname, '..', 'src', 'validation', 'generated', `${name}.mjs`), 'utf8'))
    assert.equal(artifacts.declaration, await readFile(join(import.meta.dirname, '..', 'src', 'validation', 'generated', `${name}.d.mts`), 'utf8'))
    const module = await import(`data:text/javascript,${encodeURIComponent(artifacts.code)}`)
    assert.equal(typeof module.default, 'function')
  }
})

test('AST safety allows inert helper metadata and rejects executable code generation', () => {
  assert.doesNotThrow(() => assertNoExecutableCodeGeneration(`
const helper = () => true
helper.code = 'require("ajv/runtime/helper").default'
// eval, Function, import(), and constructor are inert in comments too.
export default helper
`))
  for (const code of [
    'require("unsafe")',
    'eval("unsafe")',
    'Function("return 1")()',
    'new Function("return 1")',
    'globalThis.eval("unsafe")',
    'window["Function"]("return 1")',
    '(() => {}).constructor("return 1")()',
    'const indirect = eval',
    'import(getPath())',
  ]) {
    assert.throws(() => assertNoExecutableCodeGeneration(code), /contains executable/)
  }
})

test('the real bundling path rejects dynamic imports and indirect code generation', async () => {
  const probes = [
    ['import()', 'module.exports = (path) => import(path)'],
    ['eval', 'module.exports = () => (0, eval)("unsafe")'],
    ['Function', 'module.exports = () => Function.bind(null)("return 1")()'],
    ['Function', 'module.exports = () => Reflect.construct(Function, ["return 1"])()'],
    ['require', 'module.exports = (path) => (0, require)(path)'],
    ['constructor', 'const Code = (() => {}).constructor; module.exports = () => Code("return 1")()'],
  ]
  for (const [forbidden, commonJs] of probes) {
    await assert.rejects(
      bundleStandaloneValidator('unsafe-probe', commonJs, join(import.meta.dirname, '..')),
      new RegExp(`contains executable ${forbidden.replace(/[()]/g, '\\$&')}`),
    )
  }
})

test('validator artifacts are independent of checkout depth', { timeout: 15_000 }, async () => {
  const firstCheckout = await temporaryCheckout()
  const secondCheckout = await temporaryCheckout(true)
  const generate = `(generator, web) => generator.generateValidators(web).then((value) => Object.fromEntries(value))`
  const first = standaloneCallIn(join(firstCheckout, 'web'), generate, join(firstCheckout, 'web'))
  const second = standaloneCallIn(join(secondCheckout, 'web'), generate, join(secondCheckout, 'web'))
  assert.deepEqual(first, second)
  assert.match(first.realtime.code, /#region \\0grenadiers-validator-entry\/realtime\.cjs/)
})

test('validator output is independent of the invoking working directory', () => {
  const webDirectory = join(import.meta.dirname, '..')
  const modulePath = join(import.meta.dirname, 'validator-generator.mjs')
  const generate = `(generator, web) => generator.generateValidators(web).then((value) => Object.fromEntries(value))`
  const fromWeb = standaloneCallWithModule(webDirectory, modulePath, generate, webDirectory)
  const fromRepository = standaloneCallWithModule(join(webDirectory, '..'), modulePath, generate, webDirectory)
  assert.deepEqual(fromWeb, fromRepository)
})

test('drift detection covers generated code, declarations, and schema inputs', { timeout: 15_000 }, async () => {
  const checkout = await temporaryCheckout()
  const webDirectory = join(checkout, 'web')
  const targetDirectory = join(checkout, 'web', 'src', 'validation', 'generated')
  const generated = standaloneCallIn(webDirectory,
    `(generator, web) => generator.generateValidators(web).then((value) => Object.fromEntries(value))`,
    webDirectory,
  )
  for (const [name, artifacts] of Object.entries(generated)) {
    await writeFile(join(targetDirectory, `${name}.mjs`), artifacts.code)
    await writeFile(join(targetDirectory, `${name}.d.mts`), artifacts.declaration)
  }
  const drift = () => standaloneCallIn(webDirectory, (`(generator, web, target) => generator.findValidatorDrift(web, target)`),
    webDirectory, targetDirectory)
  assert.deepEqual(drift(), [])

  const gameplayCode = join(targetDirectory, 'gameplay-v11.mjs')
  await writeFile(gameplayCode, `${await readFile(gameplayCode, 'utf8')}\n`)
  assert.deepEqual(drift(), ['gameplay-v11.mjs'])
  await writeFile(gameplayCode, generated['gameplay-v11'].code)

  const realtimeDeclaration = join(targetDirectory, 'realtime.d.mts')
  await writeFile(realtimeDeclaration, `${await readFile(realtimeDeclaration, 'utf8')}\n`)
  assert.deepEqual(drift(), ['realtime.d.mts'])
  await writeFile(realtimeDeclaration, generated.realtime.declaration)

  const schemaPath = join(checkout, 'api', 'realtime.schema.json')
  const schema = JSON.parse(await readFile(schemaPath, 'utf8'))
  schema.$defs.Pong.properties.sequence.maximum -= 1
  await writeFile(schemaPath, JSON.stringify(schema))
  assert.deepEqual(drift(), ['realtime.mjs'])
})
