import assert from 'node:assert/strict'
import { readdir, readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'

const root = fileURLToPath(new URL('../', import.meta.url))
const read = (name) => readFile(path.join(root, name), 'utf8')
const mise = await read('mise.toml')
const nodeVersion = /^node = "([^"]+)"$/m.exec(mise)?.[1]
const rustVersion = /^rust = "([^"]+)"$/m.exec(mise)?.[1]
assert(nodeVersion && rustVersion, 'mise.toml must pin Node and Rust')
assert.equal(process.versions.node, nodeVersion, 'Activate mise or use mise exec -- scripts/gates.sh')
assert.equal((await read('.nvmrc')).trim(), nodeVersion, '.nvmrc must match mise.toml')
assert((await read('rust-toolchain.toml')).includes(`channel = "${rustVersion}"`), 'Rust toolchain must match mise.toml')
const dockerfile = await read('Dockerfile')
assert(dockerfile.includes(`node:${nodeVersion}-`), 'Docker Node version must match mise.toml')
assert(dockerfile.includes(`rust:${rustVersion}-`), 'Docker Rust version must match mise.toml')
for (const workspace of ['web', 'e2e']) {
  const pkg = JSON.parse(await read(`${workspace}/package.json`))
  assert.equal(pkg.engines?.node, nodeVersion, `${workspace} must pin the same Node version`)
  const lock = JSON.parse(await read(`${workspace}/package-lock.json`))
  assert.equal(lock.packages[''].name, pkg.name, `${workspace} lock root must match package.json`)
  for (const dependencies of [pkg.dependencies, pkg.devDependencies]) {
    for (const [name, version] of Object.entries(dependencies ?? {})) {
      assert(/^\d+\.\d+\.\d+(?:-[\w.-]+)?$/.test(version), `${workspace}: pin ${name} exactly`)
      const lockedVersion = lock.packages[`node_modules/${name}`]?.version
      assert.equal(lockedVersion, version, `${workspace}: ${name} lock must match manifest`)
    }
  }
}
const ignored = new Set(['node_modules', 'dist', 'coverage', 'target', 'playwright-report', 'test-results'])
async function checkSize(directory) {
  for (const entry of await readdir(path.join(root, directory), { withFileTypes: true })) {
    if (ignored.has(entry.name)) continue
    const relative = path.join(directory, entry.name)
    if (entry.isDirectory()) await checkSize(relative)
    else if (/\.(rs|ts|tsx)$/.test(entry.name) && !/(schema|realtime)\.ts$/.test(entry.name)) {
      const lines = (await read(relative)).trimEnd().split('\n').length
      assert(lines <= 400, `${relative}: ${lines} lines exceeds 400-line ceiling`)
    }
  }
}
for (const directory of ['core/src', 'server/src', 'web/src', 'e2e/tests']) await checkSize(directory)
console.log('Repository pins, npm lock roots, exact dependencies, and source sizes agree.')
