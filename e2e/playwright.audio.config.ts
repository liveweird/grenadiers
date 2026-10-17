import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './audio-tests',
  workers: 1,
  retries: 0,
  timeout: 60_000,
  use: { ...base.use, trace: 'off' },
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-audio-report' }]],
  outputDir: 'audio-test-results',
})
