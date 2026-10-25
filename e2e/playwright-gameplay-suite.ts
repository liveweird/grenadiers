import { defineConfig, type PlaywrightTestConfig } from '@playwright/test'
import base from './playwright.config.ts'

type GameplaySuiteOptions = {
  testDir: string
  artifactStem: string
  timeout?: number
}

export function gameplaySuiteConfig({
  testDir,
  artifactStem,
  timeout = 90_000,
}: GameplaySuiteOptions): PlaywrightTestConfig {
  return defineConfig({
    ...base,
    testDir,
    workers: 1,
    retries: 0,
    timeout,
    use: {
      ...base.use,
      // Admission frames contain stable private capabilities. Keep failure screenshots,
      // public reports and redacted summaries, but never persist native network traces.
      trace: 'off',
    },
    reporter: [['list'], ['html', {
      open: 'never', outputFolder: `playwright-${artifactStem}-report`,
    }]],
    outputDir: `${artifactStem}-test-results`,
  })
}
