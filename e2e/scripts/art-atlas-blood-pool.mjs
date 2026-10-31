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
  source, manifestJson, template, originalFrameCount: 185, appendedIds: ['effect.bloodPool'],
})
const [frame] = result.proof.frames
if (result.proof.differentBytes !== 0) throw new Error('Pixels outside effect.bloodPool changed')
if (frame.opaqueColors !== 4 || frame.invalidAlpha !== 0) {
  throw new Error('Blood pool must use exactly four opaque colors and binary alpha')
}
await mkdir(output, { recursive: false })
await Promise.all([
  writeFile(join(output, 'arena-source.png'), Buffer.from(result.pngBase64, 'base64'), { flag: 'wx' }),
  writeFile(join(output, 'arena-source.json'), result.manifestJson, { flag: 'wx' }),
  writeFile(join(output, 'pixel-proof.json'), `${JSON.stringify(result.proof, null, 2)}\n`, { flag: 'wx' }),
])
console.log(`ART_ATLAS_BLOOD_POOL_EXTENSION_PASS ${JSON.stringify(result.proof)}`)
