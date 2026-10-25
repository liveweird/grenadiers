import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './golden-flame-tests', artifactStem: 'golden-flame',
})
