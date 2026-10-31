import { readFile } from 'node:fs/promises'
import { expect, test } from '@playwright/test'

test('adopted flame poses join across both tile axes without edge seams', async ({ page }) => {
  const [png, manifestJson] = await Promise.all([
    readFile(new URL('../../assets/art/arena-source.png', import.meta.url)),
    readFile(new URL('../../assets/art/arena-source.json', import.meta.url), 'utf8'),
  ])
  await page.goto('/art-atlas.html?cli=1')
  const poses = await page.evaluate(async ({ pngBase64, manifestJson }) => {
    const bytes = Uint8Array.from(atob(pngBase64), (value) => value.charCodeAt(0))
    const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }))
    const canvas = document.createElement('canvas')
    canvas.width = bitmap.width
    canvas.height = bitmap.height
    const context = canvas.getContext('2d', { willReadFrequently: true })
    if (!context) throw new Error('Canvas 2D unavailable while checking flame joins')
    context.drawImage(bitmap, 0, 0)
    bitmap.close()
    const manifest = JSON.parse(manifestJson) as {
      frames: Array<{ id: string; sourceRect: { x: number; y: number; width: number; height: number } }>
    }
    return manifest.frames.filter(({ id }) => /^effect\.flame\.[0-5]$/.test(id))
      .map(({ id, sourceRect: r }) => {
        const edge = (x: number, y: number, width: number, height: number): number[] =>
          Array.from(context.getImageData(r.x + x, r.y + y, width, height).data)
        return {
          id, top: edge(0, 0, r.width, 1), bottom: edge(0, r.height - 1, r.width, 1),
          left: edge(0, 0, 1, r.height), right: edge(r.width - 1, 0, 1, r.height),
        }
      })
  }, { pngBase64: png.toString('base64'), manifestJson })
  expect(poses).toHaveLength(6)
  for (const pose of poses) {
    expect(pose.top, `${pose.id} vertical join`).toEqual(pose.bottom)
    expect(pose.left, `${pose.id} horizontal join`).toEqual(pose.right)
    for (const edge of [pose.top, pose.left]) {
      expect(edge.filter((alpha, index) => index % 4 === 3 && alpha === 255).length).toBeGreaterThanOrEqual(4)
    }
  }
})
