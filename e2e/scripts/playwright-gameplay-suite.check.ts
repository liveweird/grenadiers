import assert from 'node:assert/strict'
import base from '../playwright.config.ts'
import { gameplaySuiteConfig } from '../playwright-gameplay-suite.ts'

if (base.use?.launchOptions?.args?.includes('--use-angle=swiftshader') && !process.env.E2E_WORKERS) {
  assert.equal(base.workers, 1, 'the shared software renderer must default to one native worker')
}

const standard = gameplaySuiteConfig({
  testDir: './landmine-tests', artifactStem: 'landmine',
})
assert.equal(standard.testDir, './landmine-tests')
assert.equal(standard.timeout, 90_000)
assert.equal(standard.workers, 1)
assert.equal(standard.retries, 0)
assert.equal(standard.outputDir, 'landmine-test-results')
assert.deepEqual(standard.reporter, [
  ['list'], ['html', { open: 'never', outputFolder: 'playwright-landmine-report' }],
])

const audio = gameplaySuiteConfig({
  testDir: './audio-tests', artifactStem: 'audio', timeout: 60_000,
})
assert.equal(audio.testDir, './audio-tests')
assert.equal(audio.timeout, 60_000)
assert.equal(audio.outputDir, 'audio-test-results')
assert.deepEqual(audio.reporter, [
  ['list'], ['html', { open: 'never', outputFolder: 'playwright-audio-report' }],
])

assert.strictEqual(standard.projects, base.projects)
assert.strictEqual(standard.expect, base.expect)
assert.equal(standard.fullyParallel, base.fullyParallel)
assert.equal(standard.forbidOnly, base.forbidOnly)
assert.deepEqual(standard.use, { ...base.use, trace: 'off' })
assert.equal(standard.use?.trace, 'off')
assert.equal(audio.use?.trace, 'off')

console.log('PLAYWRIGHT_GAMEPLAY_SUITE_CONFIG_PASS')
