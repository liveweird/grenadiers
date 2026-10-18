import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './golden-flame-tests',
  workers: 1,
  retries: 0,
  timeout: 90_000,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-golden-flame-report' }]],
  outputDir: 'golden-flame-test-results',
})
