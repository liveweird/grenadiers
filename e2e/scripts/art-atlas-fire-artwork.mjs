#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { appendArtworkFrames } from './art-atlas-artwork-append.mjs'
import { createTemplateInBrowser } from './art-atlas-browser.mjs'
import { npmRunInvocation } from './art-atlas-launcher.mjs'
import { assertNewOutput } from './art-atlas-output.mjs'

const repository = resolve(dirname(fileURLToPath(import.meta.url)), '../..')
const webDirectory = join(repository, 'web')
const APPENDED_IDS = [
  'effect.flame.2', 'effect.flame.3', 'effect.flame.4', 'effect.flame.5',
  'effect.ember', 'effect.smoke', 'effect.ignition',
]

function valueAfter(flag) {
  const index = process.argv.indexOf(flag)
  if (index < 0 || !process.argv[index + 1]) throw new Error(`Missing ${flag}`)
  return resolve(process.cwd(), process.argv[index + 1])
}

async function run(command, args, cwd) {
  await new Promise((resolvePromise, reject) => {
    const child = spawn(command, args, { cwd, stdio: 'inherit' })
    child.once('error', reject)
    child.once('exit', (code) => code === 0
      ? resolvePromise()
      : reject(new Error(`${command} exited with ${code}`)))
  })
}

const sourcePath = valueAfter('--source')
const manifestPath = valueAfter('--manifest')
const output = valueAfter('--out')
await assertNewOutput(output)
const npm = await npmRunInvocation(['build:renderer-diagnostics'])
await run(npm.command, npm.arguments, webDirectory)
const [source, manifestJson, template] = await Promise.all([
  readFile(sourcePath), readFile(manifestPath, 'utf8'),
  createTemplateInBrowser(join(webDirectory, 'renderer-diagnostics-dist')),
])
const result = await appendArtworkFrames({
  source,
  manifestJson,
  template,
  originalFrameCount: 189,
  appendedIds: APPENDED_IDS,
  useGeneratedMetadata: true,
})
if (result.proof.oldRgbaOutsideAppended.differentBytes !== 0) {
  throw new Error('RGBA bytes outside the seven appended fire frames changed')
}
if (result.proof.newRowGutter.nonzeroBytes !== 0) {
  throw new Error('The unassigned new-row gutter must remain transparent black')
}
if (result.proof.frames.some(({ opaquePixels, invalidAlpha, opaqueColors }) =>
  opaquePixels === 0 || invalidAlpha !== 0 || opaqueColors > 7)) {
  throw new Error('Fire artwork must be visible, binary-alpha pixel art with at most seven colors')
}
await mkdir(output, { recursive: false })
await Promise.all([
  writeFile(join(output, 'arena-source.png'), Buffer.from(result.pngBase64, 'base64'), { flag: 'wx' }),
  writeFile(join(output, 'arena-source.json'), result.manifestJson, { flag: 'wx' }),
  writeFile(join(output, 'pixel-proof.json'), `${JSON.stringify(result.proof, null, 2)}\n`, { flag: 'wx' }),
])
console.log(`ART_ATLAS_FIRE_ARTWORK_EXTENSION_PASS ${JSON.stringify(result.proof)}`)
