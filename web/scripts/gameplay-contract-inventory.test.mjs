import { mkdtemp, rm, writeFile } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { pathToFileURL } from 'node:url'
import { expect, test } from 'vitest'

import { discoverGameplayContracts } from './gameplay-contract-inventory.mjs'

const supportedFilenames = [
  'gameplay.schema.json',
  ...Array.from({ length: 9 }, (_, index) => `gameplay-v${index + 3}.schema.json`),
]

function metadata(filename) {
  const version = filename === 'gameplay.schema.json' ? 2 : Number(/gameplay-v(\d+)/.exec(filename)?.[1])
  return JSON.stringify({
    $id: `https://grenadiers.local/schemas/gameplay-v${version}.json`,
    $defs: { GameplayVersion: { const: version } },
  })
}

async function withContractFiles(filenames, assertion) {
  const directory = await mkdtemp(`${tmpdir()}/grenadiers-gameplay-contracts-`)
  const directoryUrl = pathToFileURL(`${directory}/`)
  try {
    await Promise.all(filenames.map((filename) => writeFile(new URL(filename, directoryUrl), metadata(filename))))
    await assertion(directoryUrl)
  } finally {
    await rm(directory, { recursive: true, force: true })
  }
}

test('the repository inventory is exactly gameplay v2 through v11 in version order', async () => {
  const contracts = await discoverGameplayContracts()

  expect(contracts.map(({ version }) => version)).toEqual([2, 3, 4, 5, 6, 7, 8, 9, 10, 11])
  expect(contracts.map(({ sourceLabel }) => sourceLabel)).toEqual(
    supportedFilenames.map((filename) => `api/${filename}`),
  )
})

test('a malformed gameplay contract filename is rejected', async () => {
  await withContractFiles(
    [...supportedFilenames, 'gameplay-vnext.schema.json'],
    async (apiDirectory) => {
      await expect(discoverGameplayContracts({ apiDirectory })).rejects.toThrow(
        'Malformed gameplay contract filename: gameplay-vnext.schema.json',
      )
    },
  )
})

test('an unsupported gameplay contract version is rejected', async () => {
  await withContractFiles(
    [...supportedFilenames, 'gameplay-v12.schema.json'],
    async (apiDirectory) => {
      await expect(discoverGameplayContracts({ apiDirectory })).rejects.toThrow(
        'Unsupported gameplay contract version 12: gameplay-v12.schema.json',
      )
    },
  )
})

test('two filenames claiming the same gameplay version are rejected', async () => {
  await withContractFiles(
    [...supportedFilenames, 'gameplay-v2.schema.json'],
    async (apiDirectory) => {
      await expect(discoverGameplayContracts({ apiDirectory })).rejects.toThrow(
        /Duplicate gameplay contract version 2/,
      )
    },
  )
})

test('removing a supported gameplay contract is rejected', async () => {
  await withContractFiles(
    supportedFilenames.filter((filename) => filename !== 'gameplay-v7.schema.json'),
    async (apiDirectory) => {
      await expect(discoverGameplayContracts({ apiDirectory })).rejects.toThrow(
        'Missing gameplay contract versions: 7',
      )
    },
  )
})

test.each([
  ['a copied version', JSON.stringify({ $id: 'https://grenadiers.local/schemas/gameplay-v11.json', $defs: { GameplayVersion: { const: 10 } } })],
  ['a copied schema ID', JSON.stringify({ $id: 'https://grenadiers.local/schemas/gameplay-v10.json', $defs: { GameplayVersion: { const: 11 } } })],
  ['missing metadata', '{}'],
  ['a non-object schema', 'null'],
])('%s cannot be approved as a v11 contract after regeneration', async (_label, contents) => {
  await withContractFiles(supportedFilenames, async (apiDirectory) => {
    await writeFile(new URL('gameplay-v11.schema.json', apiDirectory), contents)
    await expect(discoverGameplayContracts({ apiDirectory })).rejects.toThrow(
      'Gameplay contract identity mismatch: gameplay-v11.schema.json',
    )
  })
})

test('invalid contract JSON reports its filename', async () => {
  await withContractFiles(supportedFilenames, async (apiDirectory) => {
    await writeFile(new URL('gameplay-v11.schema.json', apiDirectory), '{')
    await expect(discoverGameplayContracts({ apiDirectory })).rejects.toThrow(
      'Cannot read gameplay contract gameplay-v11.schema.json',
    )
  })
})
