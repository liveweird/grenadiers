import assert from 'node:assert/strict'
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'
import {
  checkSourceSizes,
  discoverSourceDirectories,
  sourceLineLimit,
} from './source-size.mjs'

const sourceWithLines = (lines) => Array.from({ length: lines }, () => '// source').join('\n')

async function fixture(t) {
  const root = await mkdtemp(path.join(os.tmpdir(), 'grenadiers-source-size-'))
  t.after(() => rm(root, { recursive: true, force: true }))
  for (const directory of ['core/src', 'server/src', 'web/src', 'e2e/tests']) {
    await mkdir(path.join(root, directory), { recursive: true })
  }
  return root
}

async function writeSource(root, relative, lines) {
  const target = path.join(root, relative)
  await mkdir(path.dirname(target), { recursive: true })
  await writeFile(target, sourceWithLines(lines))
}

test('accepts 400 lines and rejects 401 lines in an item test tree', async (t) => {
  const root = await fixture(t)
  await writeSource(root, 'e2e/bracelet-tests/accepted.ts', sourceLineLimit)
  await checkSourceSizes(root)

  await writeSource(root, 'e2e/bracelet-tests/rejected.ts', sourceLineLimit + 1)
  await assert.rejects(checkSourceSizes(root), /e2e[/\\]bracelet-tests[/\\]rejected\.ts: 401 lines/)
})

test('automatically discovers a newly named item test tree and nested TSX files', async (t) => {
  const root = await fixture(t)
  await writeSource(root, 'e2e/future-item-tests/nested/example.tsx', sourceLineLimit + 1)

  const directories = await discoverSourceDirectories(root)
  assert(directories.includes(path.join('e2e', 'future-item-tests')))
  await assert.rejects(checkSourceSizes(root), /future-item-tests[/\\]nested[/\\]example\.tsx/)
})

for (const [name, relative] of [
  ['core', 'core/src/oversized.rs'],
  ['server', 'server/src/oversized.rs'],
  ['web', 'web/src/oversized.tsx'],
  ['the original e2e tests tree', 'e2e/tests/oversized.ts'],
]) {
  test(`continues checking ${name} sources`, async (t) => {
    const root = await fixture(t)
    await writeSource(root, relative, sourceLineLimit + 1)
    await assert.rejects(checkSourceSizes(root), new RegExp(path.basename(relative).replace('.', '\\.')))
  })
}

test('retains generated and artifact exclusions without exempting similar source names', async (t) => {
  const root = await fixture(t)
  for (const relative of [
    'web/src/api-schema.ts',
    'web/src/events-realtime.ts',
    'web/src/node_modules/vendor.ts',
    'web/src/dist/bundle.tsx',
    'web/src/coverage/report.ts',
    'web/src/target/output.rs',
    'e2e/tests/playwright-report/report.ts',
    'e2e/tests/test-results/result.ts',
  ]) {
    await writeSource(root, relative, sourceLineLimit + 1)
  }
  await checkSourceSizes(root)

  await writeSource(root, 'web/src/api-schema-helper.ts', sourceLineLimit + 1)
  await assert.rejects(checkSourceSizes(root), /api-schema-helper\.ts/)
})

test('does not treat similarly named source directories as ignored artifacts', async (t) => {
  const root = await fixture(t)
  await writeSource(root, 'web/src/node_modules-helper/oversized.ts', sourceLineLimit + 1)
  await assert.rejects(checkSourceSizes(root), /node_modules-helper[/\\]oversized\.ts/)
})
