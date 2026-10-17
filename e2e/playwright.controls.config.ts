import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './controls-tests',
  workers: 1,
  retries: 0,
  timeout: 90_000,
  use: {
    ...base.use,
    // Personal admission frames contain capabilities. Keep screenshots and
    // bounded redacted summaries, but never retain native network traces.
    trace: 'off',
  },
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-controls-report' }]],
  outputDir: 'controls-test-results',
})
