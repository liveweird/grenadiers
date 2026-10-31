import { fileURLToPath } from 'node:url'
import { defineConfig, devices } from '@playwright/test'

const baseURL = process.env.RENDERER_DIAGNOSTICS_BASE_URL ?? 'http://127.0.0.1:5188'
const nativeLaunchOptions = process.platform === 'darwin' || process.platform === 'linux' ? {
  args: ['--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader'],
} : undefined

export default defineConfig({
  testDir: './three-arena-tests',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 60_000,
  expect: { timeout: 10_000 },
  outputDir: '.playwright/three-arena-test-results',
  reporter: 'list',
  use: {
    baseURL,
    launchOptions: nativeLaunchOptions,
    trace: 'off', screenshot: 'off', video: 'off',
  },
  webServer: process.env.RENDERER_DIAGNOSTICS_BASE_URL ? undefined : {
    command: 'npm run serve:three-arena-test',
    cwd: fileURLToPath(new URL('../web', import.meta.url)),
    url: `${baseURL}/three-arena.html`,
    reuseExistingServer: false,
    timeout: 60_000,
  },
  projects: [
    { name: 'three-arena-desktop', use: { ...devices['Desktop Chrome'], viewport: { width: 1440, height: 1050 } } },
    {
      name: 'three-arena-narrow-reduced',
      use: { ...devices['Desktop Chrome'], viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' },
    },
  ],
})
