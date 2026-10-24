#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { constants } from 'node:fs'
import { access, stat } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createTemplateInBrowser, exportInBrowser } from './art-atlas-browser.mjs'
import { npmRunInvocation } from './art-atlas-launcher.mjs'
import { assertNewOutput, writeBundle } from './art-atlas-output.mjs'

const scriptDirectory = dirname(fileURLToPath(import.meta.url))
const repository = resolve(scriptDirectory, '../..')
const webDirectory = join(repository, 'web')
const distDirectory = join(webDirectory, 'renderer-diagnostics-dist')

function usage() {
  return [
    'Usage:',
    '  art-atlas-cli.mjs template --out <directory>',
    '  art-atlas-cli.mjs export --source <png> --manifest <json> --out <directory> [--slices]',
  ].join('\n')
}

function parseArguments(argv) {
  const command = argv[0]
  const invocationDirectory = resolve(process.env.INIT_CWD ?? process.cwd())
  if (command !== 'template' && command !== 'export') throw new Error(usage())
  const options = { command, slices: false }
  for (let index = 1; index < argv.length; index += 1) {
    const argument = argv[index]
    if (argument === '--slices') {
      if (command !== 'export' || options.slices) throw new Error(`Unexpected argument ${argument}`)
      options.slices = true
      continue
    }
    if (!['--out', '--source', '--manifest'].includes(argument)) throw new Error(`Unexpected argument ${argument}\n${usage()}`)
    const key = argument.slice(2)
    const value = argv[index + 1]
    if (!value || value.startsWith('--') || options[key]) throw new Error(`Missing or repeated value for ${argument}`)
    options[key] = resolve(invocationDirectory, value)
    index += 1
  }
  if (!options.out) throw new Error('Missing required --out directory')
  if (command === 'export' && (!options.source || !options.manifest)) {
    throw new Error('Export requires --source and --manifest')
  }
  if (command === 'template' && (options.source || options.manifest)) {
    throw new Error('Template does not accept source inputs')
  }
  return options
}

async function assertInput(path, description) {
  await access(path, constants.R_OK)
  const details = await stat(path)
  if (!details.isFile()) throw new Error(`${description} must be a readable regular file`)
}

async function run(command, args, cwd) {
  await new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { cwd, stdio: 'inherit' })
    child.once('error', reject)
    child.once('exit', (code, signal) => {
      if (code === 0) resolvePromise()
      else reject(new Error(`${command} exited with ${code ?? signal}`))
    })
  })
}

async function main() {
  const options = parseArguments(process.argv.slice(2))
  await assertNewOutput(options.out)
  if (options.command === 'export') {
    await assertInput(options.source, 'Source PNG')
    await assertInput(options.manifest, 'Source manifest')
    if (options.source === options.manifest) throw new Error('Source PNG and manifest must be different files')
  }
  const npm = await npmRunInvocation(['build:renderer-diagnostics'])
  await run(npm.command, npm.arguments, webDirectory)
  const result = options.command === 'template'
    ? await createTemplateInBrowser(distDirectory)
    : await exportInBrowser(distDirectory, options.source, options.manifest, options.slices)
  await writeBundle(options.out, result.files)
  const detail = options.command === 'export'
    ? `, ${result.runtimeManifest.frames.length} frames, ${result.rgbaBytes} RGBA bytes`
    : `, ${result.manifest.frames.length} stable slots`
  console.log(`Wrote ${result.files.length} art atlas files to ${options.out}${detail}`)
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error)
  process.exitCode = 1
})
