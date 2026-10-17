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
    // Explicit visual checkpoints and failure screenshots remain. Capturing a
    // DOM/compositor snapshot per key pulse distorts real-time escape on CI.
    trace: { mode: 'retain-on-failure', snapshots: false, screenshots: false, sources: true },
  },
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-pickup-report' }]],
  outputDir: 'pickup-test-results',
})
