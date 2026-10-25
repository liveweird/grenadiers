import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { fileURLToPath } from 'node:url'
import path from 'node:path'
import { checkSourceSizes } from './source-size.mjs'

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
const cargo = await read('Cargo.toml')
const productVersion = /^\[workspace\.package\]\nversion = "(\d+\.\d+\.\d+)"$/m.exec(cargo)?.[1]
assert(productVersion, 'Cargo workspace must declare the product release version')
assert(cargo.includes(`grenadiers-core = { path = "core", version = "=${productVersion}" }`),
  'Local core dependency must match the product version')
const cargoLock = await read('Cargo.lock')
for (const name of ['grenadiers-core', 'grenadiers-server']) {
  const version = new RegExp(`^name = "${name}"\\nversion = "([^"]+)"$`, 'm').exec(cargoLock)?.[1]
  assert.equal(version, productVersion, `${name} lock must match the product version`)
}
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
await checkSourceSizes(root)
console.log('Repository pins, product version, lock roots, exact dependencies, and source sizes agree.')
