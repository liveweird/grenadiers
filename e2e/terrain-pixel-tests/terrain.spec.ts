import { expect, test } from '@playwright/test'

type TerrainPixelResult = {
  resolution: number
  comparisons: Array<{ stage: string; width: number; height: number; differentBytes: number }>
}

test('cached terrain exactly matches uncached Pixi terrain through repaint reset and context restoration',
  async ({ page }, testInfo) => {
    const expectedResolution = Number(testInfo.project.name.at(-1))
    await page.goto('/terrain-pixel.html')
    await expect(page.locator('[data-build-target]'))
      .toHaveAttribute('data-build-target', 'grenadiers-terrain-pixel-diagnostic')
    const result = await page.evaluate(async (): Promise<TerrainPixelResult> => {
      const api = (window as typeof window & {
        grenadiersTerrainPixel?: { run: () => Promise<TerrainPixelResult> }
      }).grenadiersTerrainPixel
      if (!api) throw new Error('Terrain pixel diagnostic API is unavailable')
      return api.run()
    })
    await testInfo.attach('terrain-pixel-comparisons', {
      body: JSON.stringify(result), contentType: 'application/json',
    })
    expect(result.resolution).toBe(expectedResolution)
    expect(result.comparisons.map(({ stage }) => stage))
      .toEqual(['baseline', 'repaint', 'reset', 'context-restored'])
    expect(result.comparisons.every(({ width, height }) =>
      width === 720 * expectedResolution && height === 624 * expectedResolution))
      .toBe(true)
    expect(result.comparisons.map(({ differentBytes }) => differentBytes)).toEqual([0, 0, 0, 0])
  })
