import type { Page } from '@playwright/test'

export async function arenaTilePixels(page: Page, column: number, row: number): Promise<string[]> {
  const encoded = (await page.locator('.game-arena canvas').screenshot()).toString('base64')
  return page.evaluate(async ({ encoded: png, column: x, row: y }) => {
    const image = new Image()
    image.src = `data:image/png;base64,${png}`
    await image.decode()
    const canvas = document.createElement('canvas')
    canvas.width = image.width
    canvas.height = image.height
    const context = canvas.getContext('2d')
    if (!context) throw new Error('Screenshot decoder unavailable')
    context.drawImage(image, 0, 0)
    const width = image.width / 15
    const height = image.height / 13
    const pixels = context.getImageData(Math.floor(x * width), Math.floor(y * height),
      Math.ceil(width), Math.ceil(height)).data
    const result: string[] = []
    for (let index = 0; index < pixels.length; index += 16) {
      result.push(`${pixels[index]},${pixels[index + 1]},${pixels[index + 2]},${pixels[index + 3]}`)
    }
    return result
  }, { encoded, column, row })
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
    const tileWidth = bounds.width * scale / 15
    const tileHeight = bounds.height * scale / 13
    return requested.map(({ column, row }) => {
      const data = context.getImageData(
        Math.floor(arenaX + column * tileWidth), Math.floor(arenaY + row * tileHeight),
        Math.ceil(tileWidth), Math.ceil(tileHeight),
      ).data
      const result: string[] = []
      for (let index = 0; index < data.length; index += 16) {
        result.push(`${data[index]},${data[index + 1]},${data[index + 2]},${data[index + 3]}`)
      }
      return result
    })
  }, { png: encoded, bounds: box, geometry: pageGeometry, requested: cells })
  return { screenshot, pixels }
}
