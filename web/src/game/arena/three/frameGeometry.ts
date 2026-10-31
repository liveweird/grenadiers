import { BufferGeometry, Float32BufferAttribute } from 'three'
import type { FrameAnchor, RuntimeFrame } from './contracts'

function anchorPoint(frame: RuntimeFrame, anchor: FrameAnchor): { x: number; y: number } {
  if (anchor === 'feet') return frame.footAnchor ?? frame.pivot
  if (anchor === 'center') {
    return { x: frame.sourceRect.width / 2, y: frame.sourceRect.height / 2 }
  }
  // The atlas substitutes an artist-derived pivot for paintedCenter when available.
  return frame.pivot
}

/** Creates an XY plane whose positive Y points toward the top of the source artwork. */
export function createFrameGeometry(
  frame: RuntimeFrame,
  textureSize: { width: number; height: number },
  anchor: FrameAnchor = 'pivot',
): BufferGeometry {
  const point = anchorPoint(frame, anchor)
  const left = -point.x
  const right = frame.sourceRect.width - point.x
  const top = point.y
  const bottom = point.y - frame.sourceRect.height
  const { x, y, width, height } = frame.sourceRect
  const u0 = x / textureSize.width
  const u1 = (x + width) / textureSize.width
  const v0 = 1 - (y + height) / textureSize.height
  const v1 = 1 - y / textureSize.height
  const geometry = new BufferGeometry()
  geometry.setAttribute('position', new Float32BufferAttribute([
    left, bottom, 0, right, bottom, 0, right, top, 0,
    left, bottom, 0, right, top, 0, left, top, 0,
  ], 3))
  geometry.setAttribute('uv', new Float32BufferAttribute([
    u0, v0, u1, v0, u1, v1,
    u0, v0, u1, v1, u0, v1,
  ], 2))
  geometry.computeVertexNormals()
  geometry.computeBoundingBox()
  geometry.computeBoundingSphere()
  return geometry
}
