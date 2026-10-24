import { TextureSource } from 'pixi.js'
import { describe, expect, it } from 'vitest'
import { createSourceManifest } from './manifest'
import { createAtlasRegions } from './regions'
import { buildRuntimeManifest } from './runtime'

describe('Pixi atlas regions', () => {
  it('creates untrimmed sprites on one nearest-neighbor source and owns it once', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    const source = new TextureSource({
      width: runtime.texture.width,
      height: runtime.texture.height,
      scaleMode: 'linear',
      antialias: true,
    })
    const regions = createAtlasRegions(source, runtime)
    const shadow = regions.createSprite('actor.shadow')
    const frame = runtime.frames.find(({ id }) => id === 'actor.shadow')!
    expect(shadow.texture.source).toBe(source)
    expect(shadow.texture.frame).toMatchObject(frame.sourceRect)
    expect(shadow.pivot).toMatchObject(frame.pivot)
    expect(shadow.alpha).toBeCloseTo(0.3)
    expect(source.style.scaleMode).toBe('nearest')
    expect(source.autoGenerateMipmaps).toBe(false)
    expect(source.mipLevelCount).toBe(1)
    expect(source.antialias).toBe(false)
    regions.destroy()
    regions.destroy()
    expect(source.destroyed).toBe(true)
    expect(() => regions.createSprite('actor.shadow')).toThrow('destroyed')
  })

  it('validates dimensions before allocating region textures', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    const source = new TextureSource({ width: runtime.texture.width - 1, height: runtime.texture.height })
    expect(() => createAtlasRegions(source, runtime)).toThrow('dimensions')
    expect(source.destroyed).toBe(false)
    source.destroy()
  })
})
