import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './cpu-item-tests', artifactStem: 'cpu-item', timeout: 120_000,
})
