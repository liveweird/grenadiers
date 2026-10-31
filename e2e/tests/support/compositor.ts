import type { Page } from '@playwright/test'
import { projectedCellFootprint } from './arena-projection'

export async function arenaTilePixels(page: Page, column: number, row: number): Promise<string[]> {
  const canvas = page.locator('.game-arena canvas')
  const box = await canvas.boundingBox()
  if (!box) throw new Error('Arena canvas is not visible')
  const rectangle = projectedCellFootprint({ width: box.width, height: box.height }, column, row)
  const encoded = (await canvas.screenshot()).toString('base64')
  return page.evaluate(async ({ encoded: png, bounds, requested }) => {
    const image = new Image()
    image.src = `data:image/png;base64,${png}`
    await image.decode()
    const canvas = document.createElement('canvas')
    canvas.width = image.width
    canvas.height = image.height
    const context = canvas.getContext('2d')
    if (!context) throw new Error('Screenshot decoder unavailable')
    context.drawImage(image, 0, 0)
    const scale = image.width / bounds.width
    const data = context.getImageData(
      Math.floor(requested.x * scale), Math.floor(requested.y * scale),
      Math.max(1, Math.ceil(requested.width * scale)),
      Math.max(1, Math.ceil(requested.height * scale)),
    ).data
    const result: string[] = []
    for (let index = 0; index < data.length; index += 4) {
      result.push(`${data[index]},${data[index + 1]},${data[index + 2]},${data[index + 3]}`)
    }
    return result
  }, { encoded, bounds: box, requested: rectangle })
}

export async function captureArenaTiles(
  page: Page, screenshotPath: string,
  cells: readonly { column: number, row: number }[],
): Promise<{ screenshot: Buffer, pixels: string[][] }> {
  const canvas = page.locator('.game-arena canvas')
  const box = await canvas.boundingBox()
  if (!box) throw new Error('Arena canvas is not visible')
  const pageGeometry = await page.evaluate(() => ({
    scrollX: window.scrollX,
    scrollY: window.scrollY,
    width: Math.max(document.documentElement.clientWidth, document.documentElement.scrollWidth),
  }))
  const screenshot = await page.screenshot({ path: screenshotPath, fullPage: true })
  const encoded = screenshot.toString('base64')
  const pixels = await page.evaluate(async ({ png, bounds, geometry, requested }) => {
    const image = new Image()
    image.src = `data:image/png;base64,${png}`
    await image.decode()
    const decoder = document.createElement('canvas')
    decoder.width = image.width
    decoder.height = image.height
    const context = decoder.getContext('2d')
    if (!context) throw new Error('Screenshot decoder unavailable')
    context.drawImage(image, 0, 0)
    const scale = image.width / geometry.width
    const arenaX = (bounds.x + geometry.scrollX) * scale
    const arenaY = (bounds.y + geometry.scrollY) * scale
    return requested.map(({ rectangle }) => {
      const data = context.getImageData(
        Math.floor(arenaX + rectangle.x * scale), Math.floor(arenaY + rectangle.y * scale),
        Math.max(1, Math.ceil(rectangle.width * scale)),
        Math.max(1, Math.ceil(rectangle.height * scale)),
      ).data
      const result: string[] = []
      for (let index = 0; index < data.length; index += 4) {
        result.push(`${data[index]},${data[index + 1]},${data[index + 2]},${data[index + 3]}`)
      }
      return result
    })
  }, {
    png: encoded, bounds: box, geometry: pageGeometry,
    requested: cells.map(({ column, row }) => ({
      rectangle: projectedCellFootprint({
        width: box.width, height: box.height,
      }, column, row),
    })),
  })
  return { screenshot, pixels }
}
