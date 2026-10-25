import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './diarrhea-tests', artifactStem: 'diarrhea',
})
