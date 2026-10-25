import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './short-flame-tests', artifactStem: 'short-flame',
})
