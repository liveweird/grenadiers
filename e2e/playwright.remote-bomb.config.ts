import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './remote-bomb-tests', artifactStem: 'remote-bomb',
})
