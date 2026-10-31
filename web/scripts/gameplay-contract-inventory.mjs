import { readFile, readdir } from 'node:fs/promises'
import { pathToFileURL } from 'node:url'

const supportedVersions = Object.freeze([2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18])
const supportedVersionSet = new Set(supportedVersions)
const scriptsDirectory = pathToFileURL(`${import.meta.dirname}/`)

function expectedFilename(version) {
  return version === 2 ? 'gameplay.schema.json' : `gameplay-v${version}.schema.json`
}

function versionFromFilename(filename) {
  if (filename === 'gameplay.schema.json') return 2

  const match = /^gameplay-v([1-9]\d*)\.schema\.json$/.exec(filename)
  if (!match) throw new Error(`Malformed gameplay contract filename: ${filename}`)
  return Number(match[1])
}

function validateInventory(entries) {
  const contractsByVersion = new Map()

  for (const entry of entries) {
    if (!entry.isFile()) {
      throw new Error(`Gameplay contract must be a regular file: ${entry.name}`)
    }

    const version = versionFromFilename(entry.name)
    const previous = contractsByVersion.get(version)
    if (previous) {
      throw new Error(
        `Duplicate gameplay contract version ${version}: ${previous.name} and ${entry.name}`,
      )
    }
    if (!supportedVersionSet.has(version)) {
      throw new Error(`Unsupported gameplay contract version ${version}: ${entry.name}`)
    }
    contractsByVersion.set(version, entry)
  }

  const missingVersions = supportedVersions.filter((version) => !contractsByVersion.has(version))
  if (missingVersions.length > 0) {
    throw new Error(`Missing gameplay contract versions: ${missingVersions.join(', ')}`)
  }

  for (const version of supportedVersions) {
    const filename = contractsByVersion.get(version).name
    const expected = expectedFilename(version)
    if (filename !== expected) {
      throw new Error(
        `Unexpected filename for gameplay contract version ${version}: expected ${expected}, found ${filename}`,
      )
    }
  }

  return supportedVersions.map((version) => contractsByVersion.get(version).name)
}

export async function discoverGameplayContracts({
  apiDirectory = new URL('../../api/', scriptsDirectory),
  targetDirectory = new URL('../src/game/', scriptsDirectory),
} = {}) {
  const entries = (await readdir(apiDirectory, { withFileTypes: true }))
    .filter((entry) => entry.name.startsWith('gameplay') && entry.name.endsWith('.schema.json'))
    .sort((left, right) => left.name.localeCompare(right.name))
  const filenames = validateInventory(entries)

  return Promise.all(filenames.map(async (filename, index) => {
    const version = supportedVersions[index]
    const spec = new URL(filename, apiDirectory)
    let schema
    try {
      schema = JSON.parse(await readFile(spec, 'utf8'))
    } catch (cause) {
      throw new Error(`Cannot read gameplay contract ${filename}`, { cause })
    }
    const expectedId = `https://grenadiers.local/schemas/gameplay-v${version}.json`
    if (schema?.$defs?.GameplayVersion?.const !== version || schema?.$id !== expectedId) {
      throw new Error(`Gameplay contract identity mismatch: ${filename} must declare version ${version} and $id ${expectedId}`)
    }
    return {
      version,
      spec,
      target: new URL(version === 2 ? 'schema.ts' : `v${version}/schema.ts`, targetDirectory),
      sourceLabel: `api/${filename}`,
    }
  }))
}
