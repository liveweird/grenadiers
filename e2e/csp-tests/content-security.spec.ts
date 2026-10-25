import { expect, test, type Route } from '@playwright/test'
import { observeBrowserPolicy } from '../tests/support/browser-policy'
import {
  collectBrowserFailures,
  expectEnforcedPolicy,
  FRAME_PARENT,
  FRAME_PARENT_POLICY,
  NEGATIVE_PROBE,
  POLICY,
} from './support'

test.beforeAll(() => {
  if (!process.env.E2E_CSP_HTTPS_URL) {
    throw new Error('E2E_CSP_HTTPS_URL is required for CSP browser acceptance')
  }
})

test('enforced policy permits live diagnostics and gameplay admission', async ({ context, page }, testInfo) => {
  const policy = await observeBrowserPolicy(context)
  const failures = collectBrowserFailures(page)
  const socketUrls: string[] = []
  page.on('websocket', (socket) => socketUrls.push(socket.url()))

  const response = await page.goto('/')
  expectEnforcedPolicy(response)
  await expect(page.getByRole('heading', { name: 'Grenadiers' })).toBeVisible()
  await page.getByText('Connection diagnostics', { exact: true }).click()
  await expect(page.getByText('Available', { exact: true })).toBeVisible()
  await page.getByRole('button', { name: 'Connect' }).click()
  await expect(page.getByText('Protocol v1 · up to 2 players')).toBeVisible()
  await page.getByRole('button', { name: 'Send ping' }).click()
  await expect(page.getByText('Pong 1 received')).toBeVisible()

  await page.getByRole('button', { name: 'Create room', exact: true }).click()
  await expect(page.getByLabel('Room invitation code', { exact: true }))
    .toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)

  const expectedSocketProtocol = testInfo.project.name === 'https' ? 'wss:' : 'ws:'
  expect(socketUrls).toHaveLength(2)
  expect(socketUrls.map((value) => new URL(value).protocol)).toEqual([
    expectedSocketProtocol, expectedSocketProtocol,
  ])
  expect(socketUrls.map((value) => new URL(value).pathname).sort()).toEqual([
    '/api/v1/game/ws', '/api/v1/ws',
  ])
  expect(policy.violations).toEqual([])
  expect(failures).toEqual({ console: [], page: [] })
})

test('enforced policy blocks disallowed script and connection behavior', async ({ context, page }) => {
  const policy = await observeBrowserPolicy(context)
  const forbiddenCompleted: string[] = []
  const forbiddenFailed: { error: string; url: string }[] = []
  let workerRouteReached = false
  page.on('requestfinished', (request) => {
    if (new URL(request.url()).hostname === 'example.invalid') forbiddenCompleted.push(request.url())
  })
  page.on('requestfailed', (request) => {
    if (new URL(request.url()).hostname === 'example.invalid') {
      forbiddenFailed.push({ error: request.failure()?.errorText ?? '', url: request.url() })
    }
  })
  await page.route('**/__csp_negative_probe__.js', (route) => route.fulfill({
    status: 200,
    contentType: 'text/javascript; charset=utf-8',
    body: NEGATIVE_PROBE,
  }))
  await page.route('**/__csp_forbidden_worker__.js', (route) => {
    workerRouteReached = true
    return route.fulfill({ contentType: 'text/javascript', body: "postMessage('executed')" })
  })

  const response = await page.goto('/')
  expectEnforcedPolicy(response)
  await page.evaluate(() => {
    const script = document.createElement('script')
    script.src = '/__csp_negative_probe__.js'
    document.head.append(script)
  })
  await expect.poll(() => page.evaluate(() => window.cspProbe?.finished ?? false)).toBe(true)

  expect(await page.evaluate(() => window.cspProbe)).toEqual({
    externalLoaded: true,
    inlineExecuted: false,
    evalExecuted: false,
    functionExecuted: false,
    offOriginScriptExecuted: false,
    fetchCompleted: false,
    webSocketOpened: false,
    workerExecuted: false,
    selfWorkerExecuted: false,
    finished: true,
  })
  expect(forbiddenCompleted).toEqual([])
  expect(workerRouteReached).toBe(false)
  expect(forbiddenFailed).toEqual([{
    error: 'csp', url: 'https://example.invalid/csp-script.js',
  }])
  expect(policy.violations).toEqual(expect.arrayContaining([
    'script-src-elem', 'script-src', 'connect-src', 'worker-src',
  ]))
  expect(policy.violations.filter((directive) => directive === 'connect-src')).toHaveLength(2)
  expect(policy.violations.filter((directive) => directive === 'script-src-elem').length)
    .toBeGreaterThanOrEqual(2)
})

test('response frame ancestors denies a same-origin embedding parent', async ({ page }) => {
  const parent = (route: Route, marker: string) => route.fulfill({
    status: 200,
    contentType: 'text/html; charset=utf-8',
    headers: { 'Content-Security-Policy': FRAME_PARENT_POLICY },
    body: FRAME_PARENT.replace('src="/"', `src="/?frame-proof=${marker}"`),
  })
  await page.route('**/__csp_frame_parent__.html', (route) => parent(route, 'blocked'))
  await page.route('**/__csp_frame_control__.html', (route) => parent(route, 'control'))
  const controlPolicy = POLICY.split(';').map((directive) => directive.trim())
    .filter((directive) => directive !== "frame-ancestors 'none'").join('; ')
  await page.route(/^https?:\/\/[^/]+\/\?frame-proof=(?:blocked|control)$/, async (route) => {
    const marker = new URL(route.request().url()).searchParams.get('frame-proof')
    const response = await route.fetch()
    const headers = { ...response.headers() }
    if (headers['content-security-policy'] !== POLICY) {
      throw new Error('Frame proof requires the exact application CSP')
    }
    delete headers['x-frame-options']
    if (marker === 'control') {
      headers['content-security-policy'] = headers['content-security-policy']
        .split(';').map((directive) => directive.trim())
        .filter((directive) => directive !== "frame-ancestors 'none'").join('; ')
    }
    await route.fulfill({ response, headers })
  })
  const childHeaders = new Map<string, Record<string, string>>()
  page.on('response', (response) => {
    const url = new URL(response.url())
    const marker = url.searchParams.get('frame-proof')
    if (url.pathname === '/' && marker && response.request().resourceType() === 'document') {
      childHeaders.set(marker, response.headers())
    }
  })

  await page.goto('/__csp_frame_parent__.html')
  await expect.poll(() => childHeaders.has('blocked')).toBe(true)
  expect(childHeaders.get('blocked')?.['content-security-policy']).toBe(POLICY)
  expect(childHeaders.get('blocked')?.['x-frame-options']).toBeUndefined()
  await expect.poll(() => page.frames().map((frame) => new URL(frame.url()).pathname))
    .toEqual(['/__csp_frame_parent__.html', '/'])
  expect(await page.locator('iframe').evaluate((frame) =>
    (frame as HTMLIFrameElement).contentDocument)).toBeNull()

  await page.goto('/__csp_frame_control__.html')
  await expect.poll(() => childHeaders.has('control')).toBe(true)
  expect(childHeaders.get('control')?.['content-security-policy']).toBe(controlPolicy)
  expect(childHeaders.get('control')?.['x-frame-options']).toBeUndefined()
  await expect(page.frameLocator('iframe').getByRole('heading', { name: 'Grenadiers' })).toBeVisible()
  expect(await page.locator('iframe').evaluate((frame) =>
    Boolean((frame as HTMLIFrameElement).contentDocument))).toBe(true)
})

declare global {
  interface Window {
    cspProbe?: Record<string, boolean>
  }
}
