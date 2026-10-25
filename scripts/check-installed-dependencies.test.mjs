import assert from 'node:assert/strict'
import { mkdtemp, mkdir, rm, writeFile } from 'node:fs/promises'
import os from 'node:os'
import path from 'node:path'
import test from 'node:test'
import { checkInstalledDependencies } from './check-installed-dependencies.mjs'

async function fixture(t) {
  const directory = await mkdtemp(path.join(os.tmpdir(), 'grenadiers-dependency-check-'))
  t.after(() => rm(directory, { recursive: true, force: true }))
  await writeFile(path.join(directory, 'package.json'), JSON.stringify({
    name: 'fixture', dependencies: { runtime: '1.2.3' }, devDependencies: { '@scope/tool': '4.5.6' },
  }))
  for (const [name, version] of [['runtime', '1.2.3'], ['@scope/tool', '4.5.6']]) {
    const target = path.join(directory, 'node_modules', name)
    await mkdir(target, { recursive: true })
    await writeFile(path.join(target, 'package.json'), JSON.stringify({ name, version }))
  }
  return directory
}

test('accepts matching installed runtime and scoped development dependencies', async (t) => {
  await checkInstalledDependencies(await fixture(t))
})

test('rejects a stale installed package even when the manifest is pinned', async (t) => {
  const directory = await fixture(t)
  await writeFile(path.join(directory, 'node_modules/runtime/package.json'), '{"version":"1.2.2"}')
  await assert.rejects(checkInstalledDependencies(directory), /installed runtime differs/)
})

test('rejects missing installed dependencies', async (t) => {
  const directory = await fixture(t)
  await rm(path.join(directory, 'node_modules/@scope/tool'), { recursive: true })
  await assert.rejects(checkInstalledDependencies(directory), { code: 'ENOENT' })
})
