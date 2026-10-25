import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './teleporter-tests', artifactStem: 'teleporter',
})
