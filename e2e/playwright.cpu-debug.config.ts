import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default {
  ...gameplaySuiteConfig({ testDir: './cpu-debug-tests', artifactStem: 'cpu-debug' }),
  testMatch: '**/enabled.spec.ts',
}
