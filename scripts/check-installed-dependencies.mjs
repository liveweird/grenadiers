import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import path from 'node:path'
import { fileURLToPath } from 'node:url'

const readJson = async (file) => JSON.parse(await readFile(file, 'utf8'))

export async function checkInstalledDependencies(directory) {
  const manifest = await readJson(path.join(directory, 'package.json'))
  const dependencies = { ...manifest.dependencies, ...manifest.devDependencies }
  for (const [name, expected] of Object.entries(dependencies)) {
    const installed = await readJson(path.join(directory, 'node_modules', name, 'package.json'))
    assert.equal(installed.version, expected,
      `${manifest.name}: installed ${name} differs from its pin; run npm ci in ${directory}`)
  }
}

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  const workspace = process.argv[2]
  assert(['web', 'e2e'].includes(workspace), 'usage: check-installed-dependencies.mjs web|e2e')
  const root = fileURLToPath(new URL('../', import.meta.url))
  await checkInstalledDependencies(path.join(root, workspace))
  console.log(`${workspace}: installed direct dependency versions match the manifest pins.`)
}
