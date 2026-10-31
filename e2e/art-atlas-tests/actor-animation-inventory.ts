import type { Page } from '@playwright/test'

type Source = { pngBase64: string; manifestJson: string }

export type ActorInventoryMetric = {
  appearance: string
  frameCount: number
  opaquePixels: number[]
  invalidAlpha: number
  colors: number
  coatPixels: number
  distinctFamilies: Array<{ family: string; unique: boolean }>
}

export type DeathInventoryMetric = {
  appearance: string
  frameCount: number
  opaquePixels: number[]
  invalidAlpha: number
  distinctFrameCount: number
  unexpectedColors: string[]
}

export function inspectActorInventory(
  page: Page,
  source: Source,
): Promise<{ actors: ActorInventoryMetric[]; deaths: DeathInventoryMetric[] }> {
  return page.evaluate(async (request) => {
    const binary = atob(request.pngBase64)
    const bytes = Uint8Array.from(binary, (value) => value.charCodeAt(0))
    const bitmap = await createImageBitmap(new Blob([bytes], { type: 'image/png' }))
    const canvas = document.createElement('canvas')
    canvas.width = bitmap.width
    canvas.height = bitmap.height
    const context = canvas.getContext('2d', { willReadFrequently: true })
    if (!context) throw new Error('Canvas 2D unavailable while inspecting adopted uniforms')
    context.drawImage(bitmap, 0, 0)
    bitmap.close()
    const manifest = JSON.parse(request.manifestJson) as {
      frames: Array<{ id: string; sourceRect: { x: number; y: number; width: number; height: number } }>
    }
    const differs = (left: Uint8ClampedArray, right: Uint8ClampedArray): boolean => {
      for (let index = 0; index < left.length; index += 1) {
        if (left[index] !== right[index]) return true
      }
      return false
    }
    const actors = ['one', 'two', 'three', 'four'].map((appearance) => {
      const frames = manifest.frames.filter(({ id }) =>
        new RegExp(`^actor\\.${appearance}\\.(north|east|south|west)\\.[0-3]$`).test(id))
      const colors = new Set<string>()
      const framePixels = new Map<string, Uint8ClampedArray>()
      let invalidAlpha = 0
      let coatPixels = 0
      const opaquePixels = frames.map(({ id, sourceRect }) => {
        const data = context.getImageData(
          sourceRect.x, sourceRect.y, sourceRect.width, sourceRect.height,
        ).data
        framePixels.set(id, data)
        let opaque = 0
        for (let offset = 0; offset < data.length; offset += 4) {
          const [red, green, blue, alpha] = data.slice(offset, offset + 4)
          if (alpha !== 0 && alpha !== 255) invalidAlpha += 1
          if (alpha === 0) continue
          opaque += 1
          colors.add(`${red},${green},${blue}`)
          const forest = green >= red + 12 && green >= blue + 6
          const plum = red >= green + 12 && blue >= green + 8
          if ((appearance === 'three' && forest) || (appearance === 'four' && plum)) coatPixels += 1
        }
        return opaque
      })
      const distinctFamilies = ['north', 'east', 'south', 'west'].map((direction) => {
        const poses = [0, 1, 2, 3].map((pose) =>
          framePixels.get(`actor.${appearance}.${direction}.${pose}`))
        if (poses.some((pixels) => !pixels)) throw new Error(`Incomplete ${appearance}.${direction} family`)
        const [pose0, pose1, pose2, pose3] = poses as Uint8ClampedArray[]
        return {
          family: `${appearance}.${direction}`,
          unique: differs(pose2, pose0) && differs(pose2, pose1) && differs(pose2, pose3)
            && differs(pose3, pose0) && differs(pose3, pose1),
        }
      })
      return {
        appearance, frameCount: frames.length, opaquePixels, invalidAlpha,
        colors: colors.size, coatPixels, distinctFamilies,
      }
    })
    const colorsIn = (frames: typeof manifest.frames): Set<string> => {
      const colors = new Set<string>()
      for (const { sourceRect } of frames) {
        const data = context.getImageData(
          sourceRect.x, sourceRect.y, sourceRect.width, sourceRect.height,
        ).data
        for (let offset = 0; offset < data.length; offset += 4) {
          if (data[offset + 3] !== 0) colors.add(`${data[offset]},${data[offset + 1]},${data[offset + 2]}`)
        }
      }
      return colors
    }
    const deaths = ['one', 'two', 'three', 'four', 'mob'].map((appearance) => {
      const directional = manifest.frames.filter(({ id }) => appearance === 'mob'
        ? /^mob\.(north|east|south|west)$/.test(id)
        : new RegExp(`^actor\\.${appearance}\\.(north|east|south|west)\\.[0-3]$`).test(id))
      const frames = manifest.frames.filter(({ id }) => new RegExp(
        appearance === 'mob' ? '^mob\\.death\\.[0-3]$' : `^actor\\.${appearance}\\.death\\.[0-3]$`,
      ).test(id))
      const directionalColors = colorsIn(directional)
      const deathColors = new Set<string>()
      let invalidAlpha = 0
      const framePixels = frames.map(({ sourceRect }) => {
        const data = context.getImageData(
          sourceRect.x, sourceRect.y, sourceRect.width, sourceRect.height,
        ).data
        let opaque = 0
        for (let offset = 0; offset < data.length; offset += 4) {
          const alpha = data[offset + 3]
          if (alpha !== 0 && alpha !== 255) invalidAlpha += 1
          if (alpha === 0) continue
          opaque += 1
          deathColors.add(`${data[offset]},${data[offset + 1]},${data[offset + 2]}`)
        }
        return { data, opaque }
      })
      return {
        appearance, frameCount: frames.length,
        opaquePixels: framePixels.map(({ opaque }) => opaque), invalidAlpha,
        distinctFrameCount: framePixels.filter(({ data }, index) =>
          framePixels.slice(0, index).every(({ data: prior }) => differs(data, prior))).length,
        unexpectedColors: [...deathColors].filter((color) => !directionalColors.has(color)),
      }
    })
    return { actors, deaths }
  }, source)
}
