import { fileURLToPath } from 'node:url'
import { defineConfig, devices } from '@playwright/test'

const baseURL = process.env.RENDERER_DIAGNOSTICS_BASE_URL ?? 'http://127.0.0.1:5188'
const linuxLaunchOptions = process.platform === 'linux' ? {
  args: ['--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader'],
} : undefined

export default defineConfig({
  testDir: './terrain-pixel-tests',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 30_000,
  reporter: 'list',
  use: {
    baseURL,
    launchOptions: linuxLaunchOptions,
    trace: 'off', screenshot: 'off', video: 'off',
  },
  webServer: process.env.RENDERER_DIAGNOSTICS_BASE_URL ? undefined : {
    command: 'npm run preview:renderer-diagnostics',
    cwd: fileURLToPath(new URL('../web', import.meta.url)),
    url: baseURL,
    reuseExistingServer: false,
    timeout: 30_000,
  },
  projects: [1, 2].map((deviceScaleFactor) => ({
    name: `terrain-dpr-${deviceScaleFactor}`,
    use: { ...devices['Desktop Chrome'], deviceScaleFactor },
  })),
})
