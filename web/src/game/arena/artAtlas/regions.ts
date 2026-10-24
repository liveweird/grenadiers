import { Rectangle, Sprite, Texture, type TextureSource } from 'pixi.js'
import type { RuntimeArtManifest } from './types'
import { assertValidRuntimeManifest } from './validate'

export interface AtlasRegions {
  createSprite(id: string): Sprite
  destroy(): void
}

export function createAtlasRegions(source: TextureSource, manifest: RuntimeArtManifest): AtlasRegions {
  assertValidRuntimeManifest(manifest)
  if (source.pixelWidth !== manifest.texture.width || source.pixelHeight !== manifest.texture.height) {
    throw new RangeError('texture source dimensions do not match the runtime manifest')
  }
  source.style.scaleMode = 'nearest'
  source.autoGenerateMipmaps = false
  source.antialias = false
  if (source.mipLevelCount !== 1) throw new RangeError('atlas texture source must not contain mip levels')
  const frames = new Map(manifest.frames.map((frame) => [frame.id, frame]))
  const textures = new Map(manifest.frames.map((frame) => [
    frame.id,
    new Texture({
      source,
      frame: new Rectangle(
        frame.sourceRect.x, frame.sourceRect.y, frame.sourceRect.width, frame.sourceRect.height,
      ),
      label: frame.id,
    }),
  ]))
  let destroyed = false
  return {
    createSprite(id: string): Sprite {
      if (destroyed) throw new Error('atlas regions have been destroyed')
      const resolved = manifest.aliases[id] ?? id
      const frame = frames.get(resolved)
      const texture = textures.get(resolved)
      if (!frame || !texture) throw new RangeError(`unknown atlas frame ${id}`)
      const sprite = new Sprite(texture)
      sprite.pivot.set(frame.pivot.x, frame.pivot.y)
      sprite.alpha = frame.opacity ?? 1
      return sprite
    },
    destroy(): void {
      if (destroyed) return
      destroyed = true
      for (const texture of textures.values()) texture.destroy(false)
      textures.clear()
      source.destroy()
    },
  }
}
