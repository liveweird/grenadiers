import { defineConfig, devices } from '@playwright/test'
import base, { BASE_URL } from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './latency-tests',
  workers: 1,
  retries: 0,
  timeout: 60_000,
  reporter: [['list'], ['html', { open: 'never' }]],
  use: {
    ...base.use,
    baseURL: BASE_URL,
    trace: 'off',
  },
  projects: [{ name: 'chromium-ordered-delay', use: { ...devices['Desktop Chrome'] } }],
})
