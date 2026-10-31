import { describe, expect, it } from 'vitest'
import type { RuntimeFrame } from './contracts'
import { createFrameGeometry } from './frameGeometry'

const frame: RuntimeFrame = {
  id: 'actor.one.south.0',
  sourceRect: { x: 20, y: 10, width: 8, height: 12 },
  localBounds: { x: -3, y: -4, width: 8, height: 12 },
  pivot: { x: 3, y: 4 },
  footAnchor: { x: 3, y: 10 },
}

describe('Three atlas frame geometry', () => {
  it('preserves atlas orientation while converting screen-down artwork to Y-up', () => {
    const geometry = createFrameGeometry(frame, { width: 40, height: 40 })
    expect(Array.from(geometry.getAttribute('position').array)).toEqual([
      -3, -8, 0, 5, -8, 0, 5, 4, 0, -3, -8, 0, 5, 4, 0, -3, 4, 0,
    ])
    const uv = Array.from(geometry.getAttribute('uv').array)
    const expected = [
      0.5, 0.45, 0.7, 0.45, 0.7, 0.75, 0.5, 0.45, 0.7, 0.75, 0.5, 0.75,
    ]
    expected.forEach((value, index) => expect(uv[index]).toBeCloseTo(value))
    const normals = Array.from(geometry.getAttribute('normal').array)
    const expectedNormals = [
      0, 0, 1, 0, 0, 1, 0, 0, 1,
      0, 0, 1, 0, 0, 1, 0, 0, 1,
    ]
    expectedNormals.forEach((value, index) => expect(normals[index]).toBeCloseTo(value))
  })

  it('supports feet and center anchors without changing UVs', () => {
    const feet = createFrameGeometry(frame, { width: 40, height: 40 }, 'feet')
    const center = createFrameGeometry(frame, { width: 40, height: 40 }, 'center')
    const paintedFallback = createFrameGeometry(frame, { width: 40, height: 40 }, 'paintedCenter')
    expect(Array.from(feet.getAttribute('position').array).slice(0, 6)).toEqual([-3, -2, 0, 5, -2, 0])
    expect(Array.from(center.getAttribute('position').array).slice(0, 6)).toEqual([-4, -6, 0, 4, -6, 0])
    expect(Array.from(paintedFallback.getAttribute('position').array))
      .toEqual(Array.from(createFrameGeometry(frame, { width: 40, height: 40 }).getAttribute('position').array))
    expect(Array.from(feet.getAttribute('uv').array)).toEqual(Array.from(center.getAttribute('uv').array))
  })
})
