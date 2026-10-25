import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './landmine-tests', artifactStem: 'landmine',
})
