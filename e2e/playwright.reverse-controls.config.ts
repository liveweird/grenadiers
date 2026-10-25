import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './reverse-controls-tests', artifactStem: 'reverse-controls',
})
