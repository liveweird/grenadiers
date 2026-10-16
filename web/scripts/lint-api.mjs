import fs from 'node:fs'
import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { pathToFileURL } from 'node:url'
import spectralCore from '@stoplight/spectral-core'
import spectralParsers from '@stoplight/spectral-parsers'
import spectralRulesetBundler from '@stoplight/spectral-ruleset-bundler/with-loader'

const { Document, Spectral } = spectralCore
const { Yaml } = spectralParsers
const { bundleAndLoadRuleset } = spectralRulesetBundler

const defaultSpec = resolve(import.meta.dirname, '../../api/openapi.json')
const defaultRuleset = resolve(import.meta.dirname, '../../api-guidelines/spectral.yaml')
const severityNames = ['error', 'warning', 'info', 'hint']

let rulesetPromise

function loadDefaultRuleset() {
  rulesetPromise ??= bundleAndLoadRuleset(defaultRuleset, { fs, fetch })
  return rulesetPromise
}

export async function lintApi(source, sourceName = defaultSpec) {
  const spectral = new Spectral()
  spectral.setRuleset(await loadDefaultRuleset())
  return spectral.run(new Document(source, Yaml, sourceName))
}

export function formatDiagnostic(diagnostic) {
  const line = diagnostic.range.start.line + 1
  const column = diagnostic.range.start.character + 1
  const severity = severityNames[diagnostic.severity] ?? `severity-${diagnostic.severity}`
  const location = diagnostic.path.length > 0 ? ` (${diagnostic.path.join('.')})` : ''
  return `${diagnostic.source}:${line}:${column} ${severity} ${diagnostic.code}: ${diagnostic.message}${location}`
}

async function main() {
  try {
    const source = await readFile(defaultSpec, 'utf8')
    const diagnostics = await lintApi(source)
    for (const diagnostic of diagnostics) console.log(formatDiagnostic(diagnostic))
    if (diagnostics.some((diagnostic) => diagnostic.severity === 0)) process.exitCode = 1
  } catch (error) {
    console.error(error instanceof Error ? error.message : error)
    process.exitCode = 1
  }
}

if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  await main()
}
