import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './bracelet-tests', artifactStem: 'bracelet',
})
