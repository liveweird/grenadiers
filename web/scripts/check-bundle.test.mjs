import { mkdir, mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { gzipSync } from 'node:zlib'
import { describe, expect, it } from 'vitest'
import { checkBundleBudget, measureBundle } from './check-bundle.mjs'

async function withTemporaryDirectory(run) {
  const directory = await mkdtemp(join(tmpdir(), 'grenadiers-bundle-'))
  try {
    return await run(directory)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}

async function buildFixture(directory) {
  const javascript = 'export const answer = 42\n'.repeat(20)
  const css = '.arena { color: red; }\n'.repeat(20)
  await mkdir(join(directory, 'assets', 'nested'), { recursive: true })
  await Promise.all([
    writeFile(join(directory, 'index.html'), '<div id="root"></div>'),
    writeFile(join(directory, 'assets', 'app.js'), javascript),
    writeFile(join(directory, 'assets', 'nested', 'app.css'), css),
    writeFile(join(directory, 'assets', 'app.js.map'), 'ignored source map'),
    writeFile(join(directory, 'assets', 'art.png'), 'ignored image'),
  ])
  return { javascript, css }
}

describe('compressed production bundle budget', () => {
  it('recursively measures real gzip totals and ignores unrelated extensions', () => (
    withTemporaryDirectory(async (directory) => {
      const contents = await buildFixture(directory)
      const measurement = await measureBundle(directory)

      expect(measurement).toEqual({
        assets: [
          { path: 'assets/app.js', type: 'javascript', gzipBytes: gzipSync(contents.javascript).byteLength },
          { path: 'assets/nested/app.css', type: 'css', gzipBytes: gzipSync(contents.css).byteLength },
        ],
        gzipBytes: {
          javascript: gzipSync(contents.javascript).byteLength,
          css: gzipSync(contents.css).byteLength,
        },
      })
    })
  ))

  it('accepts exact ceilings and reports totals plus largest assets when exceeded', () => (
    withTemporaryDirectory(async (directory) => {
      await buildFixture(directory)
      const measurement = await measureBundle(directory)
      const exact = { gzipBytes: { ...measurement.gzipBytes } }
      expect(checkBundleBudget(measurement, exact)).toBe(measurement)

      const tooSmall = {
        gzipBytes: { javascript: measurement.gzipBytes.javascript - 1, css: 0 },
      }
      expect(() => checkBundleBudget(measurement, tooSmall)).toThrowError(
        /javascript: \d+ > \d+ bytes[\s\S]*css: \d+ > 0 bytes[\s\S]*assets\/app\.js: \d+ bytes[\s\S]*assets\/nested\/app\.css/,
      )
    })
  ))

  it('rejects missing, empty, and non-build inputs', () => (
    withTemporaryDirectory(async (directory) => {
      await expect(measureBundle(join(directory, 'missing'))).rejects.toThrow('build directory is missing')
      await expect(measureBundle(directory)).rejects.toThrow('missing index.html')

      await writeFile(join(directory, 'index.html'), '')
      await expect(measureBundle(directory)).rejects.toThrow('empty index.html')

      await writeFile(join(directory, 'index.html'), '<main></main>')
      await writeFile(join(directory, 'notes.txt'), 'not a bundle')
      await expect(measureBundle(directory)).rejects.toThrow('no JavaScript assets')

      await expect(measureBundle(join(directory, 'index.html'))).rejects.toThrow('not a directory')
    })
  ))
})
