import { defineConfig } from '@playwright/test'
import base from './playwright.config'

export default defineConfig({
  ...base,
  testDir: './invisibility-cloak-tests', workers: 1, retries: 0, timeout: 90_000,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-invisibility-cloak-report' }]],
  outputDir: 'invisibility-cloak-test-results',
})
