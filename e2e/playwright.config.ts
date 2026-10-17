import { defineConfig, devices } from '@playwright/test'

export const BASE_URL = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8085'

export default defineConfig({
  testDir: './tests',
  fullyParallel: false,
  workers: Number(process.env.E2E_WORKERS ?? 2),
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  reporter: [['list'], ['html', { open: 'never' }]],
  timeout: 30_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL: BASE_URL,
    // Keep headless Linux WebGL and compositing on the same ANGLE backend.
    // Hybrid software compositing synchronously reads back each canvas frame
    // and can delay native key events beyond the real game's turn window.
    launchOptions: process.env.CI && process.platform === 'linux'
      ? { args: ['--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader'] }
      : undefined,
    trace: 'retain-on-failure',
    screenshot: 'only-on-failure',
    video: 'off',
  },
  projects: [{ name: 'chromium', use: { ...devices['Desktop Chrome'] } }],
})
