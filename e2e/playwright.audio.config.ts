import { gameplaySuiteConfig } from './playwright-gameplay-suite.ts'

export default gameplaySuiteConfig({
  testDir: './audio-tests', artifactStem: 'audio', timeout: 60_000,
})
