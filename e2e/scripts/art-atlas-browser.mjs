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

export async function exportInBrowser(distDirectory, sourcePath, manifestPath, slices) {
  const [png, manifestJson] = await Promise.all([readFile(sourcePath), readFile(manifestPath, 'utf8')])
  if (png.byteLength > 32 * 1024 * 1024) throw new Error('Compressed source PNG exceeds the 32MiB input limit')
  if (Buffer.byteLength(manifestJson) > 1024 * 1024) throw new Error('Source manifest exceeds the 1MiB input limit')
  return withWorkbench(distDirectory, (page) => page.evaluate(async (request) => {
    if (!globalThis.grenadiersArtAtlas) throw new Error('Art atlas browser API is unavailable')
    return globalThis.grenadiersArtAtlas.exportSource(request)
  }, { pngBase64: png.toString('base64'), manifestJson, slices }))
}
