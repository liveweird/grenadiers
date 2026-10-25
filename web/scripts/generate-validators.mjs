import { mkdir, writeFile } from 'node:fs/promises'
import { generateValidators } from './validator-generator.mjs'

const targetDirectory = new URL('../src/validation/generated/', import.meta.url)
await mkdir(targetDirectory, { recursive: true })
for (const [name, generated] of await generateValidators()) {
  await Promise.all([
    writeFile(new URL(`${name}.mjs`, targetDirectory), generated.code),
    writeFile(new URL(`${name}.d.mts`, targetDirectory), generated.declaration),
  ])
}
