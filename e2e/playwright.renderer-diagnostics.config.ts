import { fileURLToPath } from 'node:url'
import { defineConfig, devices } from '@playwright/test'

export const BASE_URL = process.env.RENDERER_DIAGNOSTICS_BASE_URL ?? 'http://127.0.0.1:5188'

const cells = ['E-E', 'A-E', 'S-E', 'E-A', 'A-A', 'S-A'] as const
const linuxLaunchOptions = process.platform === 'linux' ? {
  args: ['--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader'],
} : undefined

export default defineConfig({
  testDir: './renderer-diagnostic-tests',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 120_000,
  expect: { timeout: 10_000 },
  outputDir: 'renderer-diagnostics-test-results',
  reporter: [['list'], ['html', {
    open: 'never', outputFolder: 'playwright-renderer-diagnostics-report',
  }]],
  use: {
    baseURL: BASE_URL,
    launchOptions: linuxLaunchOptions,
    trace: 'off',
    screenshot: 'off',
    video: 'off',
  },
  webServer: process.env.RENDERER_DIAGNOSTICS_BASE_URL ? undefined : {
    command: 'npm run preview:renderer-diagnostics',
    cwd: fileURLToPath(new URL('../web', import.meta.url)),
    url: BASE_URL,
    reuseExistingServer: false,
    timeout: 30_000,
  },
  projects: cells.map((cell) => ({
    name: `renderer-${cell}`,
    use: { ...devices['Desktop Chrome'] },
  })),
})
