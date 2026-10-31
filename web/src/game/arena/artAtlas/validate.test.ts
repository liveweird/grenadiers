import { describe, expect, it } from 'vitest'
import { createSourceManifest } from './manifest'
import { buildRuntimeManifest } from './runtime'
import { ArtAtlasValidationError, type ArtManifest } from './types'
import {
  assertValidSourceManifest, validateRuntimeManifest, validateSourceManifest,
} from './validate'

const cloneManifest = (): ArtManifest => structuredClone(createSourceManifest())

function sortedObjectKeys(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(sortedObjectKeys)
  if (typeof value !== 'object' || value === null) return value
  return Object.fromEntries(Object.entries(value as Record<string, unknown>)
    .sort(([left], [right]) => right.localeCompare(left))
    .map(([key, nested]) => [key, sortedObjectKeys(nested)]))
}

describe('art atlas validation', () => {
  it('accepts the canonical source and deterministic runtime packing', () => {
    const source = createSourceManifest()
    expect(validateSourceManifest(source)).toEqual([])
    const first = buildRuntimeManifest(source)
    const second = buildRuntimeManifest(source)
    expect(second).toEqual(first)
    expect(validateRuntimeManifest(first)).toEqual([])
    expect(first.frames.some(({ id }) => id.startsWith('reserved.'))).toBe(false)
    expect(first.texture).toEqual({
      width: 2040, height: 704, padding: 2, extrusion: 2,
    })
    expect(first.texture.width * first.texture.height * 4).toBe(5_744_640)
  })

  it('accepts JSON round trips that reorder object keys but retain array order', () => {
    const reordered = JSON.parse(JSON.stringify(sortedObjectKeys(createSourceManifest()))) as ArtManifest
    expect(Object.keys(reordered.frames[0].sourceRect)).not.toEqual(
      Object.keys(createSourceManifest().frames[0].sourceRect),
    )
    expect(Object.keys(reordered.frames[0].pivot)).not.toEqual(
      Object.keys(createSourceManifest().frames[0].pivot),
    )
    expect(Object.keys(reordered.animations.flame)).not.toEqual(
      Object.keys(createSourceManifest().animations.flame),
    )
    expect(validateSourceManifest(reordered)).toEqual([])
    expect(validateRuntimeManifest(buildRuntimeManifest(reordered))).toEqual([])
  })

  it('reports shifted stable coordinates, duplicate IDs, bad feet and animation references', () => {
    const manifest = cloneManifest()
    manifest.frames[0].sourceRect.x += 1
    manifest.frames[1].id = manifest.frames[0].id
    manifest.frames.find(({ id }) => id === 'actor.one.north.0')!.footAnchor = { x: 39, y: 65 }
    manifest.frames.find(({ id }) => id === 'mob.death.3')!.footAnchor = { x: 40, y: 64 }
    manifest.animations.flame.frameIds = ['effect.missing']
    manifest.animations.spark.frameDurationMs = 91
    manifest.animations['actor.one.death'].reducedMotionFrame = 'actor.one.death.0'
    const codes = validateSourceManifest(manifest).map(({ code }) => code)
    expect(codes).toEqual(expect.arrayContaining([
      'unstable_frame', 'duplicate_id', 'missing_frame', 'foot_anchor',
      'unknown_reference', 'animation_policy', 'required_animation',
    ]))
  })

  it('rejects antialiased paint in gutters and reserved slots', () => {
    const manifest = createSourceManifest()
    const data = new Uint8ClampedArray(manifest.sheet.width * manifest.sheet.height * 4)
    const first = manifest.frames[0].sourceRect
    const gutterPixel = first.y * manifest.sheet.width + first.x + first.width + 1
    data[gutterPixel * 4 + 3] = 127
    const issues = validateSourceManifest(manifest, {
      width: manifest.sheet.width, height: manifest.sheet.height, data,
    })
    expect(issues.map(({ code }) => code)).toEqual(expect.arrayContaining([
      'paint_outside_frame', 'antialiasing',
    ]))
  })

  it('throws a structured error before an invalid manifest can be consumed', () => {
    const manifest = cloneManifest()
    manifest.sheet.width = 48
    expect(() => assertValidSourceManifest(manifest)).toThrow(ArtAtlasValidationError)
    try {
      assertValidSourceManifest(manifest)
    } catch (error) {
      expect((error as ArtAtlasValidationError).issues.some(({ code }) => code === 'sheet_layout')).toBe(true)
    }
  })

  it('detects runtime packing overlap and texture budget mistakes', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    runtime.frames[1].sourceRect = { ...runtime.frames[0].sourceRect }
    runtime.texture.width = 2049
    const codes = validateRuntimeManifest(runtime).map(({ code }) => code)
    expect(codes).toEqual(expect.arrayContaining(['overlap', 'texture_budget']))
  })

  it('detects corrupted extrusion and painted outer padding', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    const data = new Uint8ClampedArray(runtime.texture.width * runtime.texture.height * 4)
    const frame = runtime.frames[0].sourceRect
    data[((frame.y * runtime.texture.width + frame.x - 1) * 4) + 3] = 255
    data[((frame.y * runtime.texture.width + frame.x - 4) * 4) + 3] = 255
    const codes = validateRuntimeManifest(runtime, {
      width: runtime.texture.width, height: runtime.texture.height, data,
    }).map(({ code }) => code)
    expect(codes).toEqual(expect.arrayContaining(['invalid_extrusion', 'paint_in_padding']))
  })

  it('checks extrusion corners and previously unsampled side padding', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    const frame = runtime.frames[0].sourceRect
    const cornerData = new Uint8ClampedArray(runtime.texture.width * runtime.texture.height * 4)
    cornerData[(((frame.y - 1) * runtime.texture.width + frame.x - 1) * 4) + 3] = 255
    expect(validateRuntimeManifest(runtime, {
      width: runtime.texture.width, height: runtime.texture.height, data: cornerData,
    }).map(({ code }) => code)).toContain('invalid_extrusion')

    const paddingData = new Uint8ClampedArray(cornerData.length)
    paddingData[(((frame.y + 1) * runtime.texture.width + frame.x - 4) * 4) + 3] = 255
    expect(validateRuntimeManifest(runtime, {
      width: runtime.texture.width, height: runtime.texture.height, data: paddingData,
    }).map(({ code }) => code)).toContain('paint_in_padding')
  })

  it('accepts opaque clamped edges and corners but rejects trailing unused paint', () => {
    const runtime = buildRuntimeManifest(createSourceManifest())
    const frame = runtime.frames[0].sourceRect
    const data = new Uint8ClampedArray(runtime.texture.width * runtime.texture.height * 4)
    for (let row = -2; row < frame.height + 2; row += 1) {
      for (let column = -2; column < frame.width + 2; column += 1) {
        const offset = ((frame.y + row) * runtime.texture.width + frame.x + column) * 4
        data.set([18, 52, 86, 255], offset)
      }
    }
    expect(validateRuntimeManifest(runtime, {
      width: runtime.texture.width, height: runtime.texture.height, data,
    })).toEqual([])

    data[data.length - 1] = 255
    expect(validateRuntimeManifest(runtime, {
      width: runtime.texture.width, height: runtime.texture.height, data,
    }).map(({ code }) => code)).toContain('paint_in_padding')
  })
})
