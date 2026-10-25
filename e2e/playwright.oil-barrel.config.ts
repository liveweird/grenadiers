import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './oil-barrel-tests', artifactStem: 'oil-barrel',
})
