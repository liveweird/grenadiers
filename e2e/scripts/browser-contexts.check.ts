import assert from 'node:assert/strict'
import type { Browser, BrowserContext, BrowserContextOptions } from '@playwright/test'
import { createResponsiveBrowserContexts } from '../tests/support/browser-contexts.ts'

type PendingContext = {
  context: BrowserContext
  resolve: (context: BrowserContext) => void
}

function contextBrowser() {
  const calls: BrowserContextOptions[] = []
  const pending: PendingContext[] = []
  const browser = {
    newContext(options: BrowserContextOptions) {
      calls.push(options)
      return new Promise<BrowserContext>((resolve) => {
        pending.push({ context: { call: pending.length } as unknown as BrowserContext, resolve })
      })
    },
  } as Browser
  return { browser, calls, pending }
}

async function verify(mobileWidth: 320 | 390) {
  const { browser, calls, pending } = contextBrowser()
  const contextsPromise = createResponsiveBrowserContexts(browser,
    mobileWidth === 320 ? undefined : { mobileWidth })

  assert.equal(pending.length, 2, 'both native contexts must start without serializing creation')
  assert.deepEqual(calls, [
    { viewport: { width: 1440, height: 1050 } },
    { viewport: { width: mobileWidth, height: 844 }, reducedMotion: 'reduce' },
  ])

  pending[1].resolve(pending[1].context)
  pending[0].resolve(pending[0].context)
  const contexts = await contextsPromise
  assert.strictEqual(contexts[0], pending[0].context,
    'the helper must return the native desktop context')
  assert.strictEqual(contexts[1], pending[1].context,
    'the helper must return the native mobile context')
}

await verify(320)
await verify(390)

console.log('RESPONSIVE_BROWSER_CONTEXTS_PASS')
