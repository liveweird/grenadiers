import { defineConfig, devices } from '@playwright/test'
import base, { BASE_URL } from './playwright.config'

const backgroundingDefaults = [
  '--disable-background-timer-throttling',
  '--disable-backgrounding-occluded-windows',
  '--disable-renderer-backgrounding',
]

export default defineConfig({
  ...base,
  testDir: './hardening-tests',
  workers: 1,
  retries: 0,
  timeout: 60_000,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    ...base.use,
    baseURL: BASE_URL,
    headless: false,
    launchOptions: {
      ...base.use?.launchOptions,
      ignoreDefaultArgs: backgroundingDefaults,
    },
    trace: 'off',
  },
  projects: [{ name: 'chromium-headed-visibility', use: { ...devices['Desktop Chrome'] } }],
})
