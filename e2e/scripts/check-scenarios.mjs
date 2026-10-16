import { readFile, readdir } from 'node:fs/promises'

const scenarioDirectory = new URL('../scenarios/', import.meta.url)
const testDirectory = new URL('../tests/', import.meta.url)

async function files(directory, suffix) {
  return (await readdir(directory)).filter((name) => name.endsWith(suffix))
}

const scenarioText = (await Promise.all((await files(scenarioDirectory, '.md')).map((name) => readFile(new URL(name, scenarioDirectory), 'utf8')))).join('\n')
const testText = (await Promise.all((await files(testDirectory, '.ts')).map((name) => readFile(new URL(name, testDirectory), 'utf8')))).join('\n')
const scenarios = [...scenarioText.matchAll(/^## Scenario: (.+)$/gm)].map((match) => match[1])
const tests = [...testText.matchAll(/test\(['"]([^'"]+)['"]/g)].map((match) => match[1])
const missingTests = scenarios.filter((name) => !tests.includes(name))
const missingScenarios = tests.filter((name) => !scenarios.includes(name))

if (missingTests.length || missingScenarios.length) {
  console.error('Scenario/test parity failed.', { missingTests, missingScenarios })
  process.exitCode = 1
} else {
  console.log(`${scenarios.length} documented scenario matches its Playwright test.`)
}
