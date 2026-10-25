import { defineConfig, devices } from '@playwright/test'

const HTTP_URL = process.env.E2E_CSP_HTTP_URL ?? 'http://127.0.0.1:8086'
const HTTPS_URL = process.env.E2E_CSP_HTTPS_URL ?? 'https://127.0.0.1:8443'
const linuxLaunchOptions = process.platform === 'linux' ? {
  args: ['--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader'],
} : undefined

function acceptanceUrl(value: string, protocol: 'http:' | 'https:', name: string): string {
  const url = new URL(value)
  if (url.protocol !== protocol || !['127.0.0.1', '[::1]', 'localhost'].includes(url.hostname)
    || url.username || url.password || url.pathname !== '/' || url.search || url.hash) {
    throw new Error(`${name} must be a credential-free ${protocol}// loopback origin`)
  }
  return url.toString()
}

export default defineConfig({
  testDir: './csp-tests',
  fullyParallel: false,
  workers: 1,
  retries: 0,
  timeout: 30_000,
  expect: { timeout: 10_000 },
  reporter: [['list']],
  use: { launchOptions: linuxLaunchOptions, trace: 'off', screenshot: 'off', video: 'off' },
  projects: [
    {
      name: 'http',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: acceptanceUrl(HTTP_URL, 'http:', 'E2E_CSP_HTTP_URL'),
      },
    },
    {
      name: 'https',
      use: {
        ...devices['Desktop Chrome'],
        baseURL: acceptanceUrl(HTTPS_URL, 'https:', 'E2E_CSP_HTTPS_URL'),
        ignoreHTTPSErrors: true,
      },
    },
  ],
})
