import { defineConfig, devices } from '@playwright/test'

export const BASE_URL = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8085'
const defaultWorkers = process.env.CI || process.platform === 'darwin' ? 1 : 2

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: Number(process.env.E2E_WORKERS ?? defaultWorkers),
  forbidOnly: Boolean(process.env.CI),
  retries: 0,
  reporter: [['list'], ['html', { open: 'never' }]],
  timeout: 30_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL: BASE_URL,
    // Keep headless macOS and CI WebGL/compositing on the same ANGLE backend.
    // Hybrid software compositing synchronously reads back each canvas frame
    // and can delay native key events beyond the real game's turn window.
    launchOptions: process.platform === 'darwin' || process.env.CI
      ? { args: ['--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader'] }
      : undefined,
    // Personal gameplay frames contain stable seat capabilities. Native trace capture
    // observes them before application/test redaction, so keep public reports
    // and failure screenshots without persisting credential-bearing traffic.
    trace: 'off',
    screenshot: process.env.E2E_CAPTURE_PROFILE === '1' ? 'off' : 'only-on-failure',
    video: 'off',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
})
