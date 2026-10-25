import assert from 'node:assert/strict'
import { readdir, readFile } from 'node:fs/promises'
import path from 'node:path'

export const sourceLineLimit = 400

const fixedSourceDirectories = ['core/src', 'server/src', 'web/src', 'e2e/tests']
const ignoredDirectories = new Set([
  'node_modules',
  'dist',
  'coverage',
  'target',
  'playwright-report',
  'test-results',
])

export async function discoverSourceDirectories(root) {
  const e2eDirectories = await readdir(path.join(root, 'e2e'), { withFileTypes: true })
  const itemTestDirectories = e2eDirectories
    .filter((entry) => entry.isDirectory() && entry.name.endsWith('-tests'))
    .map((entry) => path.join('e2e', entry.name))

  return [...fixedSourceDirectories, ...itemTestDirectories].sort()
}

export async function checkSourceSizes(root) {
  const directories = await discoverSourceDirectories(root)

  async function checkDirectory(directory) {
    for (const entry of await readdir(path.join(root, directory), { withFileTypes: true })) {
      if (ignoredDirectories.has(entry.name)) continue
      const relative = path.join(directory, entry.name)
      if (entry.isDirectory()) {
        await checkDirectory(relative)
      } else if (/\.(rs|ts|tsx)$/.test(entry.name) && !/(schema|realtime)\.ts$/.test(entry.name)) {
        const source = await readFile(path.join(root, relative), 'utf8')
        const lines = source.trimEnd().split('\n').length
        assert(lines <= sourceLineLimit,
          `${relative}: ${lines} lines exceeds ${sourceLineLimit}-line ceiling`)
      }
    }
  }

  for (const directory of directories) await checkDirectory(directory)
}
