import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './stopwatch-tests', workers: 1, retries: 0, timeout: 90_000,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-stopwatch-report' }]],
  outputDir: 'stopwatch-test-results',
})
