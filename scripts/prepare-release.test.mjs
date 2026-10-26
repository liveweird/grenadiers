import assert from 'node:assert/strict'
import { mkdtemp, mkdir, readFile, rm, writeFile } from 'node:fs/promises'
import { spawnSync } from 'node:child_process'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'
import { fileURLToPath } from 'node:url'

const script = fileURLToPath(new URL('./prepare-release.mjs', import.meta.url))
const tracked = [
  'Cargo.toml',
  'Cargo.lock',
  'CHANGELOG.md',
  'core/Cargo.toml',
  'server/Cargo.toml',
  'web/package.json',
]

const cargo = `[workspace]
members = ["core", "server"]

[workspace.package]
version = "0.3.60"
edition = "2024"

[workspace.dependencies]
external-example = "=0.3.60"
grenadiers-core = { path = "core", version = "=0.3.60" }
serde = "1.0"
`

const lock = `# generated fixture

[[package]]
name = "external-example"
version = "0.3.60"
source = "registry+https://github.com/rust-lang/crates.io-index"
checksum = "unchanged"

[[package]]
name = "grenadiers-core"
version = "0.3.60"
dependencies = [
 "external-example",
]

[[package]]
name = "grenadiers-server"
version = "0.3.60"
dependencies = [
 "grenadiers-core",
]
`

const changelog = `# Changelog

## [Unreleased]

### Fixed

- Keep a prepared release note intact.

## 0.3.60 — 2026-10-25

### Added

- Existing release history.
`

async function fixture(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'grenadiers-prepare-release-'))
  t.after(() => rm(directory, { recursive: true, force: true }))
  await mkdir(path.join(directory, 'core'))
  await mkdir(path.join(directory, 'server'))
  await mkdir(path.join(directory, 'web'))
  await Promise.all([
    writeFile(path.join(directory, 'Cargo.toml'), cargo),
    writeFile(path.join(directory, 'Cargo.lock'), lock),
    writeFile(path.join(directory, 'CHANGELOG.md'), changelog),
    writeFile(path.join(directory, 'core/Cargo.toml'), `[package]\nname = "grenadiers-core"\nversion.workspace = true\n`),
    writeFile(path.join(directory, 'server/Cargo.toml'), `[package]
name = "grenadiers-server"
version.workspace = true

[dependencies]
grenadiers-core.workspace = true
`),
    writeFile(path.join(directory, 'web/package.json'), '{"name":"private-web","version":"0.0.0"}\n'),
  ])
  return directory
}

async function snapshot(directory) {
  return Object.fromEntries(await Promise.all(tracked.map(async (file) => [
    file,
    await readFile(path.join(directory, file), 'utf8'),
  ])))
}

function run(directory, ...arguments_) {
  return spawnSync(process.execPath, [script, ...arguments_], {
    cwd: directory,
    encoding: 'utf8',
  })
}

async function replace(directory, file, before, after) {
  const target = path.join(directory, file)
  const contents = await readFile(target, 'utf8')
  assert(contents.includes(before), `fixture ${file} contains mutation target`)
  await writeFile(target, contents.replace(before, after))
}

test('default dry run reports the plan without changing any files', async (t) => {
  const directory = await fixture(t)
  const before = await snapshot(directory)
  const result = run(directory, '--version', '0.3.61', '--date', '2026-10-26')
  assert.equal(result.status, 0, result.stderr)
  assert.match(result.stdout, /Dry run: release 0\.3\.61 \(2026-10-26\)/)
  assert.match(result.stdout, /No files changed/)
  assert.deepEqual(await snapshot(directory), before)
})

test('--write changes only product versions and promotes the Unreleased notes', async (t) => {
  const directory = await fixture(t)
  const before = await snapshot(directory)
  const result = run(directory, '--date', '2026-10-26', '--write', '--version', '0.3.61')
  assert.equal(result.status, 0, result.stderr)
  assert.match(result.stdout, /Updated 3 files/)

  assert.equal(await readFile(path.join(directory, 'Cargo.toml'), 'utf8'), cargo
    .replace('version = "0.3.60"', 'version = "0.3.61"')
    .replace('version = "=0.3.60"', 'version = "=0.3.61"'))
  assert.equal(await readFile(path.join(directory, 'Cargo.lock'), 'utf8'), lock
    .replace('name = "grenadiers-core"\nversion = "0.3.60"',
      'name = "grenadiers-core"\nversion = "0.3.61"')
    .replace('name = "grenadiers-server"\nversion = "0.3.60"',
      'name = "grenadiers-server"\nversion = "0.3.61"'))
  assert.equal(await readFile(path.join(directory, 'CHANGELOG.md'), 'utf8'), `# Changelog

## [Unreleased]

## 0.3.61 — 2026-10-26

### Fixed

- Keep a prepared release note intact.

## 0.3.60 — 2026-10-25

### Added

- Existing release history.
`)
  for (const file of ['core/Cargo.toml', 'server/Cargo.toml', 'web/package.json']) {
    assert.equal(await readFile(path.join(directory, file), 'utf8'), before[file])
  }
  assert.match(await readFile(path.join(directory, 'Cargo.toml'), 'utf8'),
    /external-example = "=0\.3\.60"/)
  assert.match(await readFile(path.join(directory, 'Cargo.lock'), 'utf8'),
    /name = "external-example"\nversion = "0\.3\.60"/)
})

test('compares increasing versions above Number safe-integer precision exactly', async (t) => {
  const directory = await fixture(t)
  await replace(directory, 'Cargo.toml',
    'version = "0.3.60"', 'version = "0.9007199254740992.0"')
  await replace(directory, 'Cargo.toml',
    'version = "=0.3.60"', 'version = "=0.9007199254740992.0"')
  for (const name of ['grenadiers-core', 'grenadiers-server']) {
    await replace(directory, 'Cargo.lock',
      `name = "${name}"\nversion = "0.3.60"`,
      `name = "${name}"\nversion = "0.9007199254740992.0"`)
  }
  await replace(directory, 'CHANGELOG.md',
    '## 0.3.60 — 2026-10-25', '## 0.9007199254740992.0 — 2026-10-25')
  const before = await snapshot(directory)
  const result = run(directory, '--version', '0.9007199254740993.0', '--date', '2026-10-26')
  assert.equal(result.status, 0, result.stderr)
  assert.deepEqual(await snapshot(directory), before)
})

const invalidCases = [
  {
    name: 'a drifted lock package',
    mutate: (directory) => replace(directory, 'Cargo.lock',
      'name = "grenadiers-server"\nversion = "0.3.60"',
      'name = "grenadiers-server"\nversion = "0.3.59"'),
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write'],
    error: /not aligned/,
  },
  {
    name: 'a registry package masquerading as the local core',
    mutate: (directory) => replace(directory, 'Cargo.lock',
      'name = "grenadiers-core"\nversion = "0.3.60"',
      'name = "grenadiers-core"\nversion = "0.3.60"\nsource = "registry+example"'),
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write'],
    error: /must be a local package entry/,
  },
  {
    name: 'a pinned server core dependency instead of the workspace dependency',
    mutate: (directory) => replace(directory, 'server/Cargo.toml',
      'grenadiers-core.workspace = true', 'grenadiers-core = "=0.3.59"'),
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write'],
    error: /must use the workspace grenadiers-core dependency/,
  },
  {
    name: 'a malformed version',
    arguments: ['--version', '0.3', '--date', '2026-10-26', '--write'],
    error: /stable MAJOR\.MINOR\.PATCH/,
  },
  {
    name: 'a version component above the Cargo unsigned 64-bit limit',
    arguments: ['--version', '18446744073709551616.0.0', '--date', '2026-10-26', '--write'],
    error: /unsigned 64-bit version limit/,
  },
  {
    name: 'a non-increasing version',
    arguments: ['--version', '0.3.60', '--date', '2026-10-26', '--write'],
    error: /must be greater/,
  },
  {
    name: 'a duplicate changelog release',
    mutate: (directory) => replace(directory, 'CHANGELOG.md',
      '## 0.3.60', '## 0.3.61 — 2026-10-24\n\n- Duplicate.\n\n## 0.3.60'),
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write'],
    error: /already contains release/,
  },
  {
    name: 'an invalid calendar date',
    arguments: ['--version', '0.3.61', '--date', '2026-02-30', '--write'],
    error: /valid calendar date/,
  },
  {
    name: 'an empty Unreleased section',
    mutate: (directory) => replace(directory, 'CHANGELOG.md',
      '\n### Fixed\n\n- Keep a prepared release note intact.\n', ''),
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write'],
    error: /must not be empty/,
  },
  {
    name: 'a missing Unreleased section',
    mutate: (directory) => replace(directory, 'CHANGELOG.md', '## [Unreleased]', '## Pending'),
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write'],
    error: /exactly one Unreleased/,
  },
  {
    name: 'an unrecognized argument',
    arguments: ['--version', '0.3.61', '--date', '2026-10-26', '--write', '--publish'],
    error: /unrecognized argument/,
  },
]

for (const scenario of invalidCases) {
  test(`rejects ${scenario.name} without writing`, async (t) => {
    const directory = await fixture(t)
    await scenario.mutate?.(directory)
    const before = await snapshot(directory)
    const result = run(directory, ...scenario.arguments)
    assert.notEqual(result.status, 0)
    assert.match(result.stderr, scenario.error)
    assert.deepEqual(await snapshot(directory), before)
  })
}
