import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default {
  ...gameplaySuiteConfig({ testDir: './cpu-debug-tests', artifactStem: 'cpu-debug-disabled' }),
  testMatch: '**/disabled.spec.ts',
}
