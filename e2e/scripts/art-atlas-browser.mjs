import { createReadStream } from 'node:fs'
import { lstat, readFile } from 'node:fs/promises'
import { createServer } from 'node:http'
import { extname, relative, resolve, sep } from 'node:path'
import { chromium } from '@playwright/test'

const MIME = new Map([
  ['.css', 'text/css; charset=utf-8'],
  ['.html', 'text/html; charset=utf-8'],
  ['.js', 'text/javascript; charset=utf-8'],
  ['.json', 'application/json; charset=utf-8'],
  ['.svg', 'image/svg+xml'],
])

function staticPath(root, requestUrl) {
  const pathname = decodeURIComponent(new URL(requestUrl ?? '/', 'http://localhost').pathname)
  const candidate = resolve(root, `.${pathname}`)
  const fromRoot = relative(root, candidate)
  if (fromRoot.startsWith(`..${sep}`) || fromRoot === '..' || fromRoot.includes('\0')) return undefined
  return candidate
}

async function serveFile(root, request, response) {
  const path = staticPath(root, request.url)
  if (!path) {
    response.writeHead(400).end('Bad path')
    return
  }
  try {
    const details = await lstat(path)
    if (!details.isFile()) throw new Error('not a file')
    response.writeHead(200, {
      'Content-Type': MIME.get(extname(path)) ?? 'application/octet-stream',
      'Content-Length': details.size,
      'Cache-Control': 'no-store',
      'X-Content-Type-Options': 'nosniff',
    })
    createReadStream(path).pipe(response)
  } catch {
    response.writeHead(404).end('Not found')
  }
}

async function startStaticServer(root) {
  const server = createServer((request, response) => void serveFile(root, request, response))
  await new Promise((resolvePromise, reject) => {
    server.once('error', reject)
    server.listen(0, '127.0.0.1', resolvePromise)
  })
  const address = server.address()
  if (!address || typeof address === 'string') throw new Error('Static server did not bind a TCP port')
  return {
    url: `http://127.0.0.1:${address.port}`,
    close: () => new Promise((resolvePromise, reject) => server.close((error) => {
      if (error) reject(error)
      else resolvePromise()
    })),
  }
}

export async function withBrowserResources(resources, operation) {
  let server
  let browser
  let result
  let primaryError
  try {
    server = await resources.startServer()
    browser = await resources.launchBrowser()
    result = await operation(server, browser)
  } catch (error) {
    primaryError = error
  }
  const cleanupErrors = []
  if (browser) {
    try { await browser.close() } catch (error) { cleanupErrors.push(error) }
  }
  if (server) {
    try { await server.close() } catch (error) { cleanupErrors.push(error) }
  }
  if (primaryError) throw primaryError
  if (cleanupErrors.length > 0) throw new AggregateError(cleanupErrors, 'Art atlas browser cleanup failed')
  return result
}

async function withWorkbench(distDirectory, operation) {
  return withBrowserResources({
    startServer: () => startStaticServer(distDirectory),
    launchBrowser: () => chromium.launch({ headless: true }),
  }, async (server, browser) => {
    const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } })
    await page.goto(`${server.url}/art-atlas.html?cli=1`, { waitUntil: 'load' })
    await page.waitForFunction(() => Boolean(globalThis.grenadiersArtAtlas))
    return await operation(page)
  })
}

export async function createTemplateInBrowser(distDirectory) {
  return withWorkbench(distDirectory, (page) => page.evaluate(async () => {
    if (!globalThis.grenadiersArtAtlas) throw new Error('Art atlas browser API is unavailable')
    return globalThis.grenadiersArtAtlas.createTemplate()
  }))
}

export async function extendMaterialsInBrowser(distDirectory, sourcePath, manifestPath) {
  const [png, manifestJson] = await Promise.all([readFile(sourcePath), readFile(manifestPath, 'utf8')])
  return withWorkbench(distDirectory, (page) => page.evaluate(async (request) => {
    const api = globalThis.grenadiersArtAtlas
    if (!api) throw new Error('Art atlas browser API is unavailable')
    const originalManifest = JSON.parse(request.manifestJson)
    const template = await api.createTemplate()
    const templatePng = template.files.find(({ name }) => name === 'arena-source.png')
    const templateJson = template.files.find(({ name }) => name === 'arena-source.json')
    if (!templatePng?.base64 || !templateJson?.text) throw new Error('Template source files are unavailable')

    const decode = async (base64) => {
      const binary = atob(base64)
      const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0))
      return globalThis.createImageBitmap(new Blob([bytes], { type: 'image/png' }))
    }
    const compareFrames = (before, after, width, frames) => {
      let comparedBytes = 0
      let differentBytes = 0
      for (const frame of frames) {
        const { x, y, width: frameWidth, height } = frame.sourceRect
        for (let row = y; row < y + height; row += 1) {
          const start = (row * width + x) * 4
          const end = start + frameWidth * 4
          for (let index = start; index < end; index += 1) {
            comparedBytes += 1
            if (before[index] !== after[index]) differentBytes += 1
          }
        }
      }
      return { comparedBytes, differentBytes }
    }
    const [original, generated] = await Promise.all([decode(request.pngBase64), decode(templatePng.base64)])
    try {
      if (original.width !== 1120 || original.height !== 1792) throw new Error('Original source must be 1120 × 1792')
      if (generated.width !== 1120 || generated.height !== 1904) throw new Error('Extended template must be 1120 × 1904')
      const canvas = globalThis.document.createElement('canvas')
      canvas.width = generated.width
      canvas.height = generated.height
      const context = canvas.getContext('2d', { willReadFrequently: true })
      if (!context) throw new Error('Canvas 2D is unavailable')
      context.imageSmoothingEnabled = false
      context.drawImage(original, 0, 0)
      for (const frame of template.manifest.frames.filter(({ id }) => id.startsWith('material.'))) {
        const { x, y, width, height } = frame.sourceRect
        context.drawImage(generated, x, y, width, height, x, y, width, height)
      }

      const originalCanvas = globalThis.document.createElement('canvas')
      originalCanvas.width = original.width
      originalCanvas.height = original.height
      const originalContext = originalCanvas.getContext('2d', { willReadFrequently: true })
      if (!originalContext) throw new Error('Canvas 2D is unavailable')
      originalContext.drawImage(original, 0, 0)
      const before = originalContext.getImageData(0, 0, original.width, original.height).data
      const after = context.getImageData(0, 0, canvas.width, canvas.height).data
      const comparison = compareFrames(before, after, original.width, originalManifest.frames)
      const blob = await new Promise((resolve, reject) => canvas.toBlob(
        (value) => value ? resolve(value) : reject(new Error('Browser PNG encoding failed')),
        'image/png',
      ))
      const output = new Uint8Array(await blob.arrayBuffer())
      let binary = ''
      for (let offset = 0; offset < output.length; offset += 0x8000) {
        binary += String.fromCharCode(...output.subarray(offset, offset + 0x8000))
      }
      return {
        pngBase64: btoa(binary), manifestJson: templateJson.text,
        proof: { originalFrames: originalManifest.frames.length, ...comparison },
      }
    } finally {
      original.close()
      generated.close()
    }
  }, { pngBase64: png.toString('base64'), manifestJson }))
}

export async function exportInBrowser(distDirectory, sourcePath, manifestPath, slices) {
  const [png, manifestJson] = await Promise.all([readFile(sourcePath), readFile(manifestPath, 'utf8')])
  if (png.byteLength > 32 * 1024 * 1024) throw new Error('Compressed source PNG exceeds the 32MiB input limit')
  if (Buffer.byteLength(manifestJson) > 1024 * 1024) throw new Error('Source manifest exceeds the 1MiB input limit')
  return withWorkbench(distDirectory, (page) => page.evaluate(async (request) => {
    if (!globalThis.grenadiersArtAtlas) throw new Error('Art atlas browser API is unavailable')
    return globalThis.grenadiersArtAtlas.exportSource(request)
  }, { pngBase64: png.toString('base64'), manifestJson, slices }))
}
