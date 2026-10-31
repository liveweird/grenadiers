import { useFrame } from '@react-three/fiber'
import { useRef } from 'react'
import { Group } from 'three'
import type { PresentationDeathCue } from '../types'
import { Billboard } from './Billboard'
import type { ThreeAtlas } from './contracts'
import { unitsToGround } from './motion'
import { DEATH_ANIMATION_MS, DEATH_FRAME_MS, REDUCED_DEATH_MS } from './sceneStore'

interface DeathCardProps {
  cue: PresentationDeathCue
  atlas: ThreeAtlas
  reducedMotion: boolean
}

const framePrefix = (cue: PresentationDeathCue): string =>
  cue.appearance === 'mob' ? 'mob.death' : `actor.${cue.appearance}.death`

function DeathCard({ cue, atlas, reducedMotion }: DeathCardProps) {
  const root = useRef<Group>(null)
  const ground = unitsToGround(cue.position)
  const prefix = framePrefix(cue)
  const lifetime = reducedMotion ? REDUCED_DEATH_MS : DEATH_ANIMATION_MS
  const frameAt = (now: number): string => {
    if (reducedMotion) return `${prefix}.3`
    const frame = Math.max(0, Math.min(3, Math.floor((now - cue.bornAt) / DEATH_FRAME_MS)))
    return `${prefix}.${frame}`
  }
  useFrame(() => {
    if (!root.current) return
    const age = performance.now() - cue.bornAt
    root.current.visible = age >= 0 && age < lifetime
  })
  return <group ref={root} name={`death:${cue.key}`} position={[ground.x, 0, ground.z]}>
    <Billboard
      atlas={atlas} frameId="actor.shadow" anchor="paintedCenter"
      position={[0, 0.003, 0]} rotation={[-Math.PI / 2, 0, 0]}
      faceCamera={false} depthTest renderOrder={2} scale={1.35 / 48}
    />
    {/* Clear hazard overlays (up to order 13), while higher status cues remain visible. */}
    <Billboard
      atlas={atlas} frameId={frameAt(cue.bornAt)} anchor="feet"
      position={[0, 0, 0]} scale={1.35 / 48} frameAt={frameAt} renderOrder={14}
    />
  </group>
}

export function Deaths({ cues, atlas, reducedMotion }: {
  cues: readonly PresentationDeathCue[]
  atlas: ThreeAtlas
  reducedMotion: boolean
}) {
  return <group name="deaths">
    {cues.map((cue) => <DeathCard
      key={cue.key} cue={cue} atlas={atlas} reducedMotion={reducedMotion}
    />)}
  </group>
}
