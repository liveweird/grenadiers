import { readFile } from 'node:fs/promises'
import { expect, test } from '@playwright/test'
import { inspectSpikePitInventory } from './actor-animation-inventory'

async function committedSource() {
  const [png, manifestJson] = await Promise.all([
    readFile(new URL('../../assets/art/arena-source.png', import.meta.url)),
    readFile(new URL('../../assets/art/arena-source.json', import.meta.url), 'utf8'),
  ])
  return { pngBase64: png.toString('base64'), manifestJson }
}

test('adopted spike-pit phases are distinct binary-alpha bounded-palette tiles', async ({ page }) => {
  await page.goto('/art-atlas.html?cli=1')
  const metrics = await inspectSpikePitInventory(page, await committedSource())
  expect(metrics.map(({ id }) => id)).toEqual([
    'terrain.spikePit.idle', 'terrain.spikePit.warning', 'terrain.spikePit.raised',
  ])
  expect(metrics.every(({ opaquePixels, invalidAlpha, colors }) =>
    opaquePixels === 48 * 48 && invalidAlpha === 0 && colors >= 6 && colors <= 12),
  JSON.stringify(metrics)).toBe(true)
  expect(new Set(metrics.map(({ pixelSignature }) => pixelSignature)).size).toBe(3)
})

