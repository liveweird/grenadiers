import { findValidatorDrift } from './validator-generator.mjs'

const stale = await findValidatorDrift()
for (const file of stale) {
  console.error(`${file} is stale. Run cd web && npm run gen:validators.`)
}
if (stale.length > 0) process.exitCode = 1
else console.log('Standalone validators match their schemas and generator.')
