import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './pickup-tests',
  workers: 1,
  retries: 0,
  timeout: 90_000,
  use: {
    ...base.use,
    // Personal v4 admission frames must never persist in native traces. Explicit
    // checkpoints, failure screenshots and redacted public summaries remain.
    trace: 'off',
  },
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-pickup-report' }]],
  outputDir: 'pickup-test-results',
})
