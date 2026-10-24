import type { ArtManifest, PixelBuffer, RuntimeArtManifest } from './types'

const PNG_SIGNATURE = [137, 80, 78, 71, 13, 10, 26, 10] as const
const MAX_RGBA_BYTES = 16 * 1024 * 1024

function assertPngHeader(bytes: Uint8Array): { width: number; height: number } {
  if (bytes.length > MAX_RGBA_BYTES) {
    throw new Error('Compressed PNG exceeds the 16MiB input budget')
  }
  if (bytes.length < 24 || PNG_SIGNATURE.some((value, index) => bytes[index] !== value)) {
    throw new Error('Source file is not a PNG image')
  }
  if (String.fromCharCode(...bytes.subarray(12, 16)) !== 'IHDR') {
    throw new Error('PNG must begin with an IHDR dimensions chunk')
  }
  const view = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength)
  const width = view.getUint32(16)
  const height = view.getUint32(20)
  if (width === 0 || height === 0 || width > 2048 || height > 2048
    || width * height * 4 > MAX_RGBA_BYTES) {
    throw new Error('PNG dimensions exceed the 2048px or 16MiB RGBA decode budget')
  }
  return { width, height }
}

export function pixelCanvas(pixels: PixelBuffer): HTMLCanvasElement {
  if (pixels.width * pixels.height * 4 !== pixels.data.length) {
    throw new Error('Pixel buffer length does not match its dimensions')
  }
  const canvas = document.createElement('canvas')
  canvas.width = pixels.width
  canvas.height = pixels.height
  const context = canvas.getContext('2d')
  if (!context) throw new Error('Canvas 2D is unavailable')
  context.putImageData(new ImageData(new Uint8ClampedArray(pixels.data), pixels.width, pixels.height), 0, 0)
  return canvas
}

export async function pngBytes(pixels: PixelBuffer): Promise<Uint8Array> {
  const canvas = pixelCanvas(pixels)
  const blob = await new Promise<Blob>((resolve, reject) => canvas.toBlob((value) => {
    if (value) resolve(value)
    else reject(new Error('Browser PNG encoding failed'))
  }, 'image/png'))
  return new Uint8Array(await blob.arrayBuffer())
}

export async function decodePng(bytes: Uint8Array, manifest: ArtManifest): Promise<PixelBuffer> {
  const dimensions = assertPngHeader(bytes)
  if (dimensions.width !== manifest.sheet.width || dimensions.height !== manifest.sheet.height) {
    throw new Error(`PNG must be ${manifest.sheet.width} × ${manifest.sheet.height} pixels`)
  }
  const ownedBytes = new Uint8Array(bytes)
  const bitmap = await createImageBitmap(new Blob([ownedBytes.buffer], { type: 'image/png' }))
  try {
    const canvas = document.createElement('canvas')
    canvas.width = dimensions.width
    canvas.height = dimensions.height
    const context = canvas.getContext('2d', { willReadFrequently: true })
    if (!context) throw new Error('Canvas 2D is unavailable')
    context.clearRect(0, 0, canvas.width, canvas.height)
    context.drawImage(bitmap, 0, 0)
    return { ...dimensions, data: context.getImageData(0, 0, canvas.width, canvas.height).data }
  } finally {
    bitmap.close()
  }
}

function copyPixel(operation: {
  source: PixelBuffer
  sourceAt: { x: number; y: number }
  target: Uint8ClampedArray
  targetWidth: number
  targetAt: { x: number; y: number }
}): void {
  const { source, sourceAt, target, targetAt, targetWidth } = operation
  const sourceOffset = (sourceAt.y * source.width + sourceAt.x) * 4
  const targetOffset = (targetAt.y * targetWidth + targetAt.x) * 4
  target.set(source.data.subarray(sourceOffset, sourceOffset + 4), targetOffset)
}

export function packRuntimePixels(
  sourcePixels: PixelBuffer,
  source: ArtManifest,
  runtime: RuntimeArtManifest,
): PixelBuffer {
  const output = new Uint8ClampedArray(runtime.texture.width * runtime.texture.height * 4)
  const sourceById = new Map(source.frames.map((frame) => [frame.id, frame]))
  const extrusion = runtime.texture.extrusion
  for (const targetFrame of runtime.frames) {
    const sourceFrame = sourceById.get(targetFrame.id)
    if (!sourceFrame) throw new Error(`Runtime frame ${targetFrame.id} has no source frame`)
    const { width, height } = sourceFrame.sourceRect
    for (let y = -extrusion; y < height + extrusion; y += 1) {
      for (let x = -extrusion; x < width + extrusion; x += 1) {
        copyPixel({
          source: sourcePixels,
          sourceAt: {
            x: sourceFrame.sourceRect.x + Math.max(0, Math.min(width - 1, x)),
            y: sourceFrame.sourceRect.y + Math.max(0, Math.min(height - 1, y)),
          },
          target: output,
          targetWidth: runtime.texture.width,
          targetAt: { x: targetFrame.sourceRect.x + x, y: targetFrame.sourceRect.y + y },
        })
      }
    }
  }
  return { width: runtime.texture.width, height: runtime.texture.height, data: output }
}

export function slicePixels(
  pixels: PixelBuffer, x: number, y: number, width: number, height: number,
): PixelBuffer {
  const output = new Uint8ClampedArray(width * height * 4)
  for (let row = 0; row < height; row += 1) {
    const start = ((y + row) * pixels.width + x) * 4
    output.set(pixels.data.subarray(start, start + width * 4), row * width * 4)
  }
  return { width, height, data: output }
}

export function bytesToBase64(bytes: Uint8Array): string {
  let result = ''
  for (let offset = 0; offset < bytes.length; offset += 0x8000) {
    result += String.fromCharCode(...bytes.subarray(offset, offset + 0x8000))
  }
  return btoa(result)
}

export function base64ToBytes(value: string): Uint8Array {
  const binary = atob(value)
  const result = new Uint8Array(binary.length)
  for (let index = 0; index < binary.length; index += 1) result[index] = binary.charCodeAt(index)
  return result
}
