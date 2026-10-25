import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './stopwatch-tests', artifactStem: 'stopwatch',
})
