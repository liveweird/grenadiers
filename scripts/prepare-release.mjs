#!/usr/bin/env node
import { readFile, writeFile } from 'node:fs/promises'
import path from 'node:path'

const FILES = ['Cargo.toml', 'Cargo.lock', 'CHANGELOG.md']
const CARGO_VERSION_COMPONENT_MAX = 18_446_744_073_709_551_615n

function usage() {
  return 'usage: node scripts/prepare-release.mjs --version MAJOR.MINOR.PATCH --date YYYY-MM-DD [--write]'
}

function parseArguments(argv) {
  const result = { write: false }
  for (let index = 0; index < argv.length; index += 1) {
    const argument = argv[index]
    if (argument === '--write') {
      if (result.write) throw new Error('--write may be specified only once')
      result.write = true
    } else if (argument === '--version' || argument === '--date') {
      const key = argument.slice(2)
      const value = argv[index + 1]
      if (!value || value.startsWith('--')) throw new Error(`${argument} requires a value`)
      if (result[key]) throw new Error(`${argument} may be specified only once`)
      result[key] = value
      index += 1
    } else {
      throw new Error(`unrecognized argument: ${argument}`)
    }
  }
  if (!result.version || !result.date) throw new Error(`--version and --date are required\n${usage()}`)
  return result
}

function parseVersion(value, label) {
  const match = /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.exec(value)
  if (!match) throw new Error(`${label} must be a stable MAJOR.MINOR.PATCH version`)
  const components = match.slice(1).map(BigInt)
  if (components.some((component) => component > CARGO_VERSION_COMPONENT_MAX)) {
    throw new Error(`${label} components must fit Cargo's unsigned 64-bit version limit`)
  }
  return components
}

function compareVersions(left, right) {
  for (let index = 0; index < 3; index += 1) {
    if (left[index] < right[index]) return -1
    if (left[index] > right[index]) return 1
  }
  return 0
}

function validateDate(value) {
  const match = /^(\d{4})-(\d{2})-(\d{2})$/.exec(value)
  if (!match) throw new Error('--date must use YYYY-MM-DD')
  const [year, month, day] = match.slice(1).map(Number)
  const parsed = new Date(Date.UTC(year, month - 1, day))
  if (parsed.getUTCFullYear() !== year || parsed.getUTCMonth() !== month - 1 ||
      parsed.getUTCDate() !== day) {
    throw new Error('--date must be a valid calendar date')
  }
}

function locateSection(text, heading, file) {
  const expression = new RegExp(`^\\[${heading.replaceAll('.', '\\.')}\\]\\r?$`, 'gm')
  const matches = [...text.matchAll(expression)]
  if (matches.length !== 1) throw new Error(`${file} must contain exactly one [${heading}] section`)
  const match = matches[0]
  const start = match.index + match[0].length
  const next = /^\[/m.exec(text.slice(start))
  return { start, end: next ? start + next.index : text.length, body: text.slice(start, next ? start + next.index : text.length) }
}

function oneMatch(text, expression, message) {
  const matches = [...text.matchAll(expression)]
  if (matches.length !== 1) throw new Error(message)
  return matches[0]
}

function prepareCargo(text, nextVersion) {
  const workspace = locateSection(text, 'workspace.package', 'Cargo.toml')
  const version = oneMatch(workspace.body, /^version = "([^"]+)"$/gm,
    'Cargo.toml must contain exactly one workspace package version')
  const dependencies = locateSection(text, 'workspace.dependencies', 'Cargo.toml')
  const core = oneMatch(dependencies.body,
    /^grenadiers-core = \{ path = "core", version = "=([^"]+)" \}$/gm,
    'Cargo.toml must contain exactly one local grenadiers-core dependency')
  if (version[1] !== core[1]) {
    throw new Error('Cargo.toml workspace and local grenadiers-core versions are not aligned')
  }
  const versionStart = workspace.start + version.index + version[0].indexOf(version[1])
  const coreStart = dependencies.start + core.index + core[0].indexOf(core[1])
  const replacements = [[versionStart, version[1]], [coreStart, core[1]]].sort((a, b) => b[0] - a[0])
  let updated = text
  for (const [start, current] of replacements) {
    updated = `${updated.slice(0, start)}${nextVersion}${updated.slice(start + current.length)}`
  }
  return { currentVersion: version[1], updated }
}

function validateMemberManifest(text, name, file, requireCoreDependency = false) {
  const packageSection = locateSection(text, 'package', file)
  const packageName = oneMatch(packageSection.body, /^name = "([^"]+)"$/gm,
    `${file} must contain exactly one package name`)[1]
  if (packageName !== name || !/^version\.workspace = true$/m.test(packageSection.body)) {
    throw new Error(`${file} must define ${name} with the workspace version`)
  }
  if (requireCoreDependency) {
    const dependencies = locateSection(text, 'dependencies', file)
    const declarations = [...dependencies.body.matchAll(/^grenadiers-core(?:\.workspace)?\s*=.*$/gm)]
    if (declarations.length !== 1 || declarations[0][0] !== 'grenadiers-core.workspace = true') {
      throw new Error(`${file} must use the workspace grenadiers-core dependency`)
    }
  }
}

function prepareLock(text, currentVersion, nextVersion) {
  const starts = [...text.matchAll(/^\[\[package\]\]\r?$/gm)].map((match) => match.index)
  const blocks = starts.map((start, index) => ({
    index: start,
    contents: text.slice(start, starts[index + 1] ?? text.length),
  }))
  const replacements = []
  for (const name of ['grenadiers-core', 'grenadiers-server']) {
    const candidates = blocks.filter((block) => new RegExp(`^name = "${name}"$`, 'm').test(block.contents))
    if (candidates.length !== 1) throw new Error(`Cargo.lock must contain exactly one ${name} package`)
    const block = candidates[0]
    if (/^(source|checksum) = /m.test(block.contents)) {
      throw new Error(`Cargo.lock ${name} must be a local package entry`)
    }
    const version = oneMatch(block.contents, /^version = "([^"]+)"$/gm,
      `Cargo.lock ${name} must contain exactly one version`)
    if (version[1] !== currentVersion) {
      throw new Error(`Cargo.lock ${name} version is not aligned with Cargo.toml`)
    }
    const start = block.index + version.index + version[0].indexOf(version[1])
    replacements.push([start, version[1]])
  }
  let updated = text
  for (const [start, current] of replacements.sort((a, b) => b[0] - a[0])) {
    updated = `${updated.slice(0, start)}${nextVersion}${updated.slice(start + current.length)}`
  }
  return updated
}

function prepareChangelog(text, currentVersion, nextVersion, date) {
  const unreleased = [...text.matchAll(/^## \[Unreleased\][ \t]*\r?$/gm)]
  if (unreleased.length !== 1) throw new Error('CHANGELOG.md must contain exactly one Unreleased section')
  if (new RegExp(`^## \\[?v?${nextVersion.replaceAll('.', '\\.')}\\]?\\s`, 'm').test(text)) {
    throw new Error(`CHANGELOG.md already contains release ${nextVersion}`)
  }
  const headingEnd = unreleased[0].index + unreleased[0][0].length
  const following = /^## /m.exec(text.slice(headingEnd))
  const sectionEnd = following ? headingEnd + following.index : text.length
  const notes = text.slice(headingEnd, sectionEnd).trim()
  if (!notes) throw new Error('CHANGELOG.md Unreleased section must not be empty')
  const currentHeading = /^## \[?v?(\d+\.\d+\.\d+)\]?\s/m.exec(text.slice(sectionEnd))
  if (!currentHeading || currentHeading[1] !== currentVersion) {
    throw new Error('CHANGELOG.md latest release is not aligned with Cargo.toml')
  }
  const replacement = `\n\n## ${nextVersion} — ${date}\n\n${notes}\n\n`
  return `${text.slice(0, headingEnd)}${replacement}${text.slice(sectionEnd)}`
}

async function main() {
  const options = parseArguments(process.argv.slice(2))
  const next = parseVersion(options.version, '--version')
  validateDate(options.date)
  const root = process.cwd()
  const [cargo, lock, changelog, core, server] = await Promise.all([
    ...FILES.map((file) => readFile(path.join(root, file), 'utf8')),
    readFile(path.join(root, 'core/Cargo.toml'), 'utf8'),
    readFile(path.join(root, 'server/Cargo.toml'), 'utf8'),
  ])
  const preparedCargo = prepareCargo(cargo, options.version)
  const current = parseVersion(preparedCargo.currentVersion, 'current Cargo version')
  if (compareVersions(next, current) <= 0) {
    throw new Error(`--version must be greater than current version ${preparedCargo.currentVersion}`)
  }
  validateMemberManifest(core, 'grenadiers-core', 'core/Cargo.toml')
  validateMemberManifest(server, 'grenadiers-server', 'server/Cargo.toml', true)
  const outputs = [
    preparedCargo.updated,
    prepareLock(lock, preparedCargo.currentVersion, options.version),
    prepareChangelog(changelog, preparedCargo.currentVersion, options.version, options.date),
  ]
  console.log(`${options.write ? 'Writing' : 'Dry run:'} release ${options.version} (${options.date})`)
  for (const file of FILES) console.log(`  ${file}`)
  if (!options.write) {
    console.log('No files changed. Pass --write to apply this plan.')
    return
  }
  await Promise.all(FILES.map((file, index) => writeFile(path.join(root, file), outputs[index])))
  console.log(`Updated ${FILES.length} files; no checks, Git operations, tags, or publishing were run.`)
}

main().catch((error) => {
  console.error(`prepare-release: ${error.message}`)
  process.exitCode = 1
})
