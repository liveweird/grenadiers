import { chromium } from '@playwright/test'

function templateFile(template, name, field) {
  const value = template.files.find((file) => file.name === name)?.[field]
  if (!value) throw new Error(`Template ${name} is unavailable`)
  return value
}

/**
 * Copy an append-only frame prefix from a generated template into an adopted sheet.
 * The returned proof compares decoded RGBA bytes, including transparent RGB bytes,
 * everywhere outside the explicitly appended frame rectangles.
 */
export async function appendArtworkFrames({
  source, manifestJson, template, originalFrameCount, appendedIds,
  expectedWidth = 1120, expectedHeight = 2128,
}) {
  const templateBase64 = templateFile(template, 'arena-source.png', 'base64')
  const templateJson = templateFile(template, 'arena-source.json', 'text')
  const browser = await chromium.launch({ headless: true })
  try {
    const page = await browser.newPage()
    return await page.evaluate(async (request) => {
      const originalManifest = JSON.parse(request.manifestJson)
      const generatedManifest = JSON.parse(request.templateJson)
      const targetFrameCount = request.originalFrameCount + request.appendedIds.length
      if (originalManifest.frames.length !== request.originalFrameCount) {
        throw new Error(`Original manifest must contain ${request.originalFrameCount} frames`)
      }
      if (generatedManifest.frames.length < targetFrameCount) {
        throw new Error(`Generated manifest must contain at least ${targetFrameCount} frames`)
      }
      if (JSON.stringify(originalManifest.frames)
        !== JSON.stringify(generatedManifest.frames.slice(0, request.originalFrameCount))) {
        throw new Error('Generated manifest moved or changed an established frame')
      }
      const appendedFrames = generatedManifest.frames.slice(request.originalFrameCount, targetFrameCount)
      if (JSON.stringify(appendedFrames.map(({ id }) => id)) !== JSON.stringify(request.appendedIds)) {
        throw new Error(`Generated manifest does not append ${request.appendedIds.join(', ')}`)
      }

      const decode = async (base64) => {
        const binary = atob(base64)
        const bytes = Uint8Array.from(binary, (character) => character.charCodeAt(0))
        return globalThis.createImageBitmap(new Blob([bytes], { type: 'image/png' }))
      }
      const toBase64 = (bytes) => {
        let binary = ''
        for (let offset = 0; offset < bytes.length; offset += 0x8000) {
          binary += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000))
        }
        return btoa(binary)
      }
      const inside = (x, y, rectangles) => rectangles.some((rectangle) =>
        x >= rectangle.x && x < rectangle.x + rectangle.width
        && y >= rectangle.y && y < rectangle.y + rectangle.height)
      const changedChannels = (before, after, offset) => [0, 1, 2, 3]
        .filter((channel) => before[offset + channel] !== after[offset + channel]).length
      const compareOutside = (before, after, width, height, rectangles) => {
        let comparedBytes = 0
        let differentBytes = 0
        for (let y = 0; y < height; y += 1) {
          for (let x = 0; x < width; x += 1) {
            if (inside(x, y, rectangles)) continue
            const offset = (y * width + x) * 4
            comparedBytes += 4
            differentBytes += changedChannels(before, after, offset)
          }
        }
        return { comparedBytes, differentBytes }
      }
      const inspectFrame = (after, sheetWidth, { id, sourceRect }) => {
        let opaquePixels = 0
        let invalidAlpha = 0
        const colors = new Set()
        for (let y = sourceRect.y; y < sourceRect.y + sourceRect.height; y += 1) {
          for (let x = sourceRect.x; x < sourceRect.x + sourceRect.width; x += 1) {
            const offset = (y * sheetWidth + x) * 4
            const alpha = after[offset + 3]
            if (alpha !== 0 && alpha !== 255) invalidAlpha += 1
            if (alpha === 255) {
              opaquePixels += 1
              colors.add(`${after[offset]},${after[offset + 1]},${after[offset + 2]}`)
            }
          }
        }
        return { id, opaquePixels, invalidAlpha, opaqueColors: colors.size, sourceRect }
      }
      const inspect = (before, after, width, height, frames) => {
        const rectangles = frames.map(({ sourceRect }) => sourceRect)
        return {
          ...compareOutside(before, after, width, height, rectangles),
          frames: frames.map((frame) => inspectFrame(after, width, frame)),
        }
      }

      const [original, generated] = await Promise.all([
        decode(request.sourceBase64), decode(request.templateBase64),
      ])
      try {
        if (original.width !== request.expectedWidth || original.height !== request.expectedHeight) {
          throw new Error(`Original source must be ${request.expectedWidth} x ${request.expectedHeight}`)
        }
        if (generated.width !== original.width || generated.height !== original.height) {
          throw new Error('Generated template changed the physical canvas')
        }
        const canvas = globalThis.document.createElement('canvas')
        canvas.width = original.width
        canvas.height = original.height
        const context = canvas.getContext('2d', { willReadFrequently: true })
        if (!context) throw new Error('Canvas 2D is unavailable')
        context.imageSmoothingEnabled = false
        context.drawImage(original, 0, 0)
        for (const { sourceRect: rectangle } of appendedFrames) {
          const { x, y, width, height } = rectangle
          context.clearRect(x, y, width, height)
          context.drawImage(generated, x, y, width, height, x, y, width, height)
        }
        const blob = await new Promise((resolve, reject) => canvas.toBlob(
          (value) => value ? resolve(value) : reject(new Error('Browser PNG encoding failed')),
          'image/png',
        ))
        const pngBase64 = toBase64(new Uint8Array(await blob.arrayBuffer()))
        const result = await decode(pngBase64)
        try {
          const beforeCanvas = globalThis.document.createElement('canvas')
          beforeCanvas.width = original.width
          beforeCanvas.height = original.height
          const beforeContext = beforeCanvas.getContext('2d', { willReadFrequently: true })
          if (!beforeContext) throw new Error('Canvas 2D is unavailable')
          beforeContext.drawImage(original, 0, 0)
          const before = beforeContext.getImageData(0, 0, original.width, original.height).data
          context.clearRect(0, 0, canvas.width, canvas.height)
          context.drawImage(result, 0, 0)
          const after = context.getImageData(0, 0, canvas.width, canvas.height).data
          const comparison = inspect(before, after, canvas.width, canvas.height, appendedFrames)
          const outputManifest = { ...generatedManifest, frames: generatedManifest.frames.slice(0, targetFrameCount) }
          return {
            pngBase64,
            manifestJson: `${JSON.stringify(outputManifest, null, 2)}\n`,
            proof: {
              originalFrames: originalManifest.frames.length,
              generatedFrames: outputManifest.frames.length,
              ...comparison,
            },
          }
        } finally { result.close() }
      } finally {
        original.close()
        generated.close()
      }
    }, {
      sourceBase64: source.toString('base64'), manifestJson,
      templateBase64, templateJson, originalFrameCount, appendedIds,
      expectedWidth, expectedHeight,
    })
  } finally {
    await browser.close()
  }
}
