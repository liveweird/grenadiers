import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './wall-pass-tests', artifactStem: 'wall-pass',
})
