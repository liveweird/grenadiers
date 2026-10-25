import { fileURLToPath } from 'node:url'

import { scenarioSpecMap } from './scenario-map.mjs'
import { checkScenarioContract } from './scenario-contract.mjs'

const rootDirectory = fileURLToPath(new URL('../', import.meta.url))

try {
  const count = await checkScenarioContract({ rootDirectory, mapping: scenarioSpecMap })
  console.log(`${count} documented scenarios match their mapped Playwright specs.`)
} catch (error) {
  console.error(error instanceof Error ? error.message : error)
  process.exitCode = 1
}
