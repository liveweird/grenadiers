import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './invisible-bombs-tests', artifactStem: 'invisible-bombs',
})
