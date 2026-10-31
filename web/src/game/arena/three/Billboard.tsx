import { useFrame } from '@react-three/fiber'
import { useEffect, useMemo, useRef } from 'react'
import {
  DoubleSide, Mesh, MeshBasicMaterial, NormalBlending,
} from 'three'
import type { FrameAnchor, ThreeAtlas } from './contracts'

type Point3 = readonly [number, number, number]
export interface BillboardProps {
  atlas: ThreeAtlas
  frameId: string
  anchor?: FrameAnchor
  position?: Point3
  rotation?: Point3
  scale?: number
  opacity?: number
  depthTest?: boolean
  renderOrder?: number
  faceCamera?: boolean
  frameAt?: (now: number) => string
  opacityAt?: (now: number) => number
}

/** A pixel-crisp atlas card which borrows, but never disposes, shared atlas resources. */
export function Billboard({
  atlas, frameId, anchor = 'center', position = [0, 0, 0], rotation,
  scale = 1 / 48, opacity = 1, depthTest = false, renderOrder = 10,
  faceCamera = true, frameAt, opacityAt,
}: BillboardProps) {
  const mesh = useRef<Mesh>(null)
  const material = useMemo(() => new MeshBasicMaterial({
    alphaTest: 0.08,
    blending: NormalBlending,
    depthTest,
    depthWrite: false,
    map: atlas.texture,
    opacity: (atlas.frame(frameId).opacity ?? 1) * opacity,
    side: DoubleSide,
    toneMapped: false,
    transparent: true,
  }), [atlas, depthTest, frameId, opacity])

  useEffect(() => () => material.dispose(), [material])
  useFrame(({ camera }) => {
    const node = mesh.current
    if (!node) return
    const now = performance.now()
    if (faceCamera) node.quaternion.copy(camera.quaternion)
    if (frameAt) {
      const nextFrame = frameAt(now)
      if (node.geometry !== atlas.geometry(nextFrame, anchor)) {
        node.geometry = atlas.geometry(nextFrame, anchor)
      }
    }
    const animatedOpacity = opacityAt?.(now) ?? opacity
    const metadataOpacity = atlas.frame(frameAt?.(now) ?? frameId).opacity ?? 1
    const nodeMaterial = node.material as MeshBasicMaterial
    nodeMaterial.opacity = metadataOpacity * animatedOpacity
  })

  return <mesh
    name={`billboard:${frameId}`}
    ref={mesh}
    dispose={null}
    geometry={atlas.geometry(frameId, anchor)}
    material={material}
    position={position}
    {...(rotation ? { rotation } : {})}
    scale={scale}
    renderOrder={renderOrder}
    frustumCulled={false}
  />
}
