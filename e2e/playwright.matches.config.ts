import { defineConfig } from '@playwright/test'
import base from './playwright.pickups.config'

export default defineConfig({
  ...base,
  testDir: './match-tests',
  timeout: 180_000,
  reporter: [['list'], ['html', { open: 'never', outputFolder: 'playwright-match-report' }]],
  outputDir: 'match-test-results',
})
