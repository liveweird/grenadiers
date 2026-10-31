import { useFrame } from '@react-three/fiber'
import { useLayoutEffect, useRef, useState } from 'react'
import { Group } from 'three'
import type { ActorView } from './actorView'
import { actorFrameId, cueOpacity } from './actorView'
import { Billboard } from './Billboard'
import type { ThreeAtlas } from './contracts'
import type { MotionSegment, MotionTiming } from './motion'
import { nextMotion, sampleMotion } from './motion'

interface ActorVisualProps {
  atlas: ThreeAtlas
  view: ActorView
  timing: MotionTiming
  reducedMotion: boolean
}

const ACTOR_SCALE = 1.35 / 48
const DEFAULT_CUE_OFFSET = [0, 25 / 48] as const

export function ActorVisual({ atlas, view, timing, reducedMotion }: ActorVisualProps) {
  const root = useRef<Group>(null)
  const [initialPosition] = useState(() => [
    view.groundPosition.x, 0, view.groundPosition.z,
  ] as const)
  const motion = useRef<MotionSegment>({
    from: view.groundPosition,
    to: view.groundPosition,
    receivedAt: timing.receivedAt,
    duration: timing.duration,
    snap: true,
  })
  const previous = useRef<Pick<ActorView, 'teleportMarker' | 'frozen'> | null>(null)
  const moving = useRef(false)

  useLayoutEffect(() => {
    const node = root.current
    if (!node) return
    const old = previous.current
    const teleported = old !== null && old.teleportMarker !== view.teleportMarker
    const enteringFreeze = view.frozen && old !== null && !old.frozen
    motion.current = nextMotion(
      { x: node.position.x, z: node.position.z },
      view.groundPosition,
      timing,
      reducedMotion || teleported || enteringFreeze,
    )
    moving.current = view.moving && !motion.current.snap
      && (motion.current.from.x !== motion.current.to.x || motion.current.from.z !== motion.current.to.z)
    previous.current = view
    if (motion.current.snap) node.position.set(view.groundPosition.x, 0, view.groundPosition.z)
  }, [reducedMotion, timing, view])

  useFrame(() => {
    const node = root.current
    if (!node) return
    const point = sampleMotion(motion.current, performance.now())
    node.position.set(point.x, 0, point.z)
  })

  const animated = timing.running && !reducedMotion
  return <group ref={root} name={`actor:${view.id}`}
    position={initialPosition}>
    <Billboard
      atlas={atlas}
      frameId="actor.shadow"
      anchor="paintedCenter"
      position={[0, 0.003, 0]}
      rotation={[-Math.PI / 2, 0, 0]}
      faceCamera={false}
      depthTest
      renderOrder={2}
      scale={ACTOR_SCALE}
    />
    <Billboard
      atlas={atlas}
      frameId={actorFrameId(view, 0, false)}
      anchor="feet"
      position={[0, 0, 0]}
      scale={ACTOR_SCALE}
      frameAt={(now) => actorFrameId(view, now, animated && moving.current)}
      fireTint
    />
    {view.cues.map((cueView, index) => {
      const layout = cueView.layout
      const [offsetX, offsetY] = layout?.offset ?? DEFAULT_CUE_OFFSET
      return <Billboard
        key={`${cueView.frameId}:${index}`}
        atlas={atlas}
        frameId={cueView.frameId}
        anchor="center"
        position={[offsetX * 1.35, offsetY * 1.35, 0]}
        scale={ACTOR_SCALE * (layout?.scale ?? 1)}
        opacityAt={(now) => cueOpacity(cueView, now, animated)}
        renderOrder={11 + index}
      />
    })}
  </group>
}
