import type { Browser, BrowserContext } from '@playwright/test'

type ResponsiveBrowserContextOptions = {
  mobileWidth?: 320 | 390
}

export function createResponsiveBrowserContexts(
  browser: Browser,
  { mobileWidth = 320 }: ResponsiveBrowserContextOptions = {},
): Promise<[BrowserContext, BrowserContext]> {
  return Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({
      viewport: { width: mobileWidth, height: 844 },
      reducedMotion: 'reduce',
    }),
  ])
}
