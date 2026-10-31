#!/usr/bin/env node
import { spawn } from 'node:child_process'
import { mkdir, readFile, writeFile } from 'node:fs/promises'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { chromium } from '@playwright/test'
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

async function compositeBloodPool(source, manifestJson, template) {
  const templatePng = template.files.find(({ name }) => name === 'arena-source.png')
  const templateJson = template.files.find(({ name }) => name === 'arena-source.json')
  if (!templatePng?.base64 || !templateJson?.text) throw new Error('Template source files are unavailable')
  const browser = await chromium.launch({ headless: true })
  try {
    const page = await browser.newPage()
    return await page.evaluate(async (request) => {
      const originalManifest = JSON.parse(request.manifestJson)
      const generatedManifest = JSON.parse(request.templateJson)
      const frame = generatedManifest.frames.at(-1)
      if (frame?.id !== 'effect.bloodPool' || generatedManifest.frames.length !== 186) {
        throw new Error('Generated manifest must append effect.bloodPool at slot 185')
      }
      if (originalManifest.frames.length !== 185) throw new Error('Original manifest must contain 185 frames')
      if (JSON.stringify(originalManifest.frames) !== JSON.stringify(generatedManifest.frames.slice(0, 185))) {
        throw new Error('Generated manifest moved or changed an established frame')
      }
      const decode = async (base64) => {
        const binary = atob(base64)
        const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0))
        return globalThis.createImageBitmap(new Blob([bytes], { type: 'image/png' }))
      }
      const toBase64 = (bytes) => {
        let binary = ''
        for (let offset = 0; offset < bytes.length; offset += 0x8000) {
          binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000))
        }
        return btoa(binary)
      }
      const comparePixels = (before, after, sheetWidth, sheetHeight, rectangle) => {
        let comparedBytes = 0
        let differentBytes = 0
        let opaquePixels = 0
        const colors = new Set()
        for (let pixelY = 0; pixelY < sheetHeight; pixelY += 1) {
          for (let pixelX = 0; pixelX < sheetWidth; pixelX += 1) {
            const index = (pixelY * sheetWidth + pixelX) * 4
            const inFrame = pixelX >= rectangle.x && pixelX < rectangle.x + rectangle.width
              && pixelY >= rectangle.y && pixelY < rectangle.y + rectangle.height
            if (!inFrame) {
              comparedBytes += 4
              differentBytes += Number(before[index] !== after[index])
                + Number(before[index + 1] !== after[index + 1])
                + Number(before[index + 2] !== after[index + 2])
                + Number(before[index + 3] !== after[index + 3])
              continue
            }
            if (after[index + 3] === 255) {
              opaquePixels += 1
              colors.add(`${after[index]},${after[index + 1]},${after[index + 2]}`)
            } else if (after[index + 3] !== 0) {
              throw new Error('Blood-pool alpha must be binary')
            }
          }
        }
        return { comparedBytes, differentBytes, opaquePixels, opaqueColors: colors.size }
      }
      const [original, generated] = await Promise.all([
        decode(request.sourceBase64), decode(request.templateBase64),
      ])
      try {
        if (original.width !== 1120 || original.height !== 2128) throw new Error('Original source must be 1120 × 2128')
        if (generated.width !== original.width || generated.height !== original.height) {
          throw new Error('Generated template changed the physical canvas')
        }
        const canvas = globalThis.document.createElement('canvas')
        canvas.width = original.width
        canvas.height = original.height
        const context = canvas.getContext('2d', { willReadFrequently: true })
        if (!context) throw new Error('Canvas 2D is unavailable')
        context.imageSmoothingEnabled = false
        context.drawImage(original, 0, 0)
        const { x, y, width, height } = frame.sourceRect
        context.clearRect(x, y, width, height)
        context.drawImage(generated, x, y, width, height, x, y, width, height)
        const blob = await new Promise((resolveBlob, reject) => canvas.toBlob(
          (value) => value ? resolveBlob(value) : reject(new Error('Browser PNG encoding failed')),
          'image/png',
        ))
        const outputBase64 = toBase64(new Uint8Array(await blob.arrayBuffer()))
        const result = await decode(outputBase64)
        try {
          const beforeCanvas = globalThis.document.createElement('canvas')
          beforeCanvas.width = original.width
          beforeCanvas.height = original.height
          const beforeContext = beforeCanvas.getContext('2d', { willReadFrequently: true })
          if (!beforeContext) throw new Error('Canvas 2D is unavailable')
          beforeContext.drawImage(original, 0, 0)
          const before = beforeContext.getImageData(0, 0, original.width, original.height).data
          context.clearRect(0, 0, canvas.width, canvas.height)
          context.drawImage(result, 0, 0)
          const after = context.getImageData(0, 0, canvas.width, canvas.height).data
          const comparison = comparePixels(before, after, canvas.width, canvas.height, frame.sourceRect)
          return {
            pngBase64: outputBase64,
            manifestJson: request.templateJson,
            proof: {
              originalFrames: originalManifest.frames.length,
              generatedFrames: generatedManifest.frames.length,
              ...comparison,
              frame: frame.sourceRect,
            },
          }
        } finally {
          result.close()
        }
      } finally {
        original.close()
        generated.close()
      }
    }, {
      sourceBase64: source.toString('base64'), manifestJson,
      templateBase64: templatePng.base64, templateJson: templateJson.text,
    })
  } finally {
    await browser.close()
  }
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
const result = await compositeBloodPool(source, manifestJson, template)
if (result.proof.differentBytes !== 0) throw new Error('Pixels outside effect.bloodPool changed')
if (result.proof.opaqueColors !== 4) throw new Error('Blood pool must use exactly four opaque colors')
await mkdir(output, { recursive: false })
await Promise.all([
  writeFile(join(output, 'arena-source.png'), Buffer.from(result.pngBase64, 'base64'), { flag: 'wx' }),
  writeFile(join(output, 'arena-source.json'), result.manifestJson, { flag: 'wx' }),
  writeFile(join(output, 'pixel-proof.json'), `${JSON.stringify(result.proof, null, 2)}\n`, { flag: 'wx' }),
])
console.log(`ART_ATLAS_BLOOD_POOL_EXTENSION_PASS ${JSON.stringify(result.proof)}`)
