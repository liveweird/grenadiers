import { defineConfig } from '@playwright/test'
import base from './playwright.matches.config'

export default defineConfig({
  ...base,
  testDir: './reconnect-tests',
  timeout: 120_000,
  use: {
    ...base.use,
    // Resume capabilities stay in browser memory. Persist only explicit public
    // summaries/screenshots, rather than network traces containing credentials.
    trace: 'off',
  },
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-reconnect-report' }]],
  outputDir: 'reconnect-test-results',
})
