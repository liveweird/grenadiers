import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './shield-tests', artifactStem: 'shield',
})
