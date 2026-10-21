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
