import { readFile, readdir } from 'node:fs/promises'

const scenarioDirectory = new URL('../scenarios/', import.meta.url)
const testDirectory = new URL('../tests/', import.meta.url)

async function files(directory, suffix) {
  return (await readdir(directory)).filter((name) => name.endsWith(suffix))
}

const scenarioText = (await Promise.all((await files(scenarioDirectory, '.md')).map((name) => readFile(new URL(name, scenarioDirectory), 'utf8')))).join('\n')
const testDirectories = [
  testDirectory,
  new URL('../pickup-tests/', import.meta.url),
  new URL('../controls-tests/', import.meta.url),
  new URL('../match-tests/', import.meta.url),
  new URL('../reconnect-tests/', import.meta.url),
  new URL('../hardening-tests/', import.meta.url),
  new URL('../latency-tests/', import.meta.url),
  new URL('../audio-tests/', import.meta.url),
  new URL('../heart-tests/', import.meta.url),
  new URL('../shield-tests/', import.meta.url),
  new URL('../golden-flame-tests/', import.meta.url),
  new URL('../short-fuse-tests/', import.meta.url),
  new URL('../short-flame-tests/', import.meta.url),
  new URL('../long-fuse-tests/', import.meta.url),
  new URL('../reverse-controls-tests/', import.meta.url),
  new URL('../diarrhea-tests/', import.meta.url),
  new URL('../slow-foot-tests/', import.meta.url),
  new URL('../piercing-bomb-tests/', import.meta.url),
  new URL('../terrain-pixel-tests/', import.meta.url),
]
const testText = (await Promise.all(testDirectories.map(async (directory) =>
  (await Promise.all((await files(directory, '.ts')).map((name) => readFile(new URL(name, directory), 'utf8')))).join('\n'),
))).join('\n')
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
