import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './slow-foot-tests', artifactStem: 'slow-foot',
})
