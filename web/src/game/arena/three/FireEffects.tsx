import { useFrame } from '@react-three/fiber'
import { useRef } from 'react'
import { Group } from 'three'
import { Billboard } from './Billboard'
import type { ThreeAtlas } from './contracts'
import type { FirePresentation, FireVisual } from './firePresentation'
import { FIRE_IGNITION_MS } from './firePresentation'

const POSES = [0, 2, 1, 3, 4, 5]

function Ignition({ cell, atlas }: { cell: FireVisual; atlas: ThreeAtlas }) {
  const root = useRef<Group>(null)
  useFrame(() => {
    if (root.current) root.current.visible = cell.bornAt !== null
      && performance.now() - cell.bornAt >= 0 && performance.now() - cell.bornAt < FIRE_IGNITION_MS
  })
  return <group ref={root} name={`ignition:${cell.key}`}
    position={[cell.column + 0.5, 0.48, cell.row + 0.5]}>
    <Billboard atlas={atlas} frameId="effect.ignition" additive depthTest renderOrder={12}
      scale={0.88 / 48} opacityAt={(now) => cell.bornAt === null
        ? 0 : Math.max(0, 1 - (now - cell.bornAt) / FIRE_IGNITION_MS) * 0.55} />
  </group>
}

function FireParticle({ atlas, x, z, index, smoke }: {
  atlas: ThreeAtlas; x: number; z: number; index: number; smoke: boolean
}) {
  const root = useRef<Group>(null)
  const duration = smoke ? 1250 : 760
  useFrame(() => {
    if (!root.current) return
    const progress = ((performance.now() + index * 239) % duration) / duration
    root.current.position.set(
      x + Math.sin(index * 2.7) * progress * 0.25,
      0.5 + progress * (smoke ? 0.8 : 0.65),
      z + Math.cos(index * 1.9) * progress * 0.12,
    )
    const scale = smoke ? 0.3 + progress * 0.45 : 0.75
    root.current.scale.setScalar(scale)
  })
  return <group ref={root} name={smoke ? 'fire-smoke' : 'fire-ember'} position={[x, 0.5, z]}>
    <Billboard atlas={atlas} frameId={smoke ? 'effect.smoke' : 'effect.ember'} depthTest
      additive={!smoke} renderOrder={12} opacityAt={(now) => {
        const progress = ((now + index * 239) % duration) / duration
        return Math.sin(progress * Math.PI) * (smoke ? 0.2 : 0.85)
      }} />
  </group>
}

/** Flame footprints remain authoritative; all drift, age and ignition are cosmetic. */
export function FireEffects({ fire, atlas, running, reducedMotion }: {
  fire: FirePresentation; atlas: ThreeAtlas; running: boolean; reducedMotion: boolean
}) {
  const animated = running && !reducedMotion
  const bursts = reducedMotion ? [] : fire.cells.filter(({ bornAt }) => bornAt !== null).slice(0, 8)
  return <group name="fire-effects">
    {fire.cells.map((cell) => <Billboard key={cell.key} atlas={atlas} frameId="effect.flame.0"
      position={[cell.column + 0.5, 0.5, cell.row + 0.5]} renderOrder={cell.oil ? 12 : 10}
      frameAt={(now) => `effect.flame.${animated ? POSES[Math.floor(now / 80) % POSES.length] : 0}`} />)}
    {bursts.map((cell) => <Ignition key={cell.key} cell={cell} atlas={atlas} />)}
    {animated && fire.lighting.anchors.map(({ id, position: [x, , z] }, index) => <group key={id}>
      <FireParticle atlas={atlas} x={x} z={z} index={index * 3} smoke />
      <FireParticle atlas={atlas} x={x} z={z} index={index * 3 + 1} smoke={false} />
      <FireParticle atlas={atlas} x={x} z={z} index={index * 3 + 2} smoke={false} />
    </group>)}
  </group>
}
