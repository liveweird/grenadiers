import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './diarrhea-tests', workers: 1, retries: 0, timeout: 90_000,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-diarrhea-report' }]],
  outputDir: 'diarrhea-test-results',
})
