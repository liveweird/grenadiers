import { useFrame } from '@react-three/fiber'
import { useLayoutEffect, useRef, useState } from 'react'
import { Group } from 'three'
import type { SnapshotMessage } from '../types'
import { createActorView } from './actorView'
import { ActorVisual } from './ActorVisual'
import { Billboard } from './Billboard'
import type { ThreeAtlas } from './contracts'
import { bombView, cellCenter, cellKey, cloudPosition, mobPosition, pickupFrame } from './entityView'
import type { GroundPosition, MotionSegment, MotionTiming } from './motion'
import { nextMotion, sampleMotion } from './motion'

interface EntitiesProps {
  snapshot: SnapshotMessage
  atlas: ThreeAtlas
  timing: MotionTiming
  reducedMotion: boolean
}
interface MovingSpriteProps {
  name: string
  atlas: ThreeAtlas
  frameId: string
  target: GroundPosition
  timing: MotionTiming
  reducedMotion: boolean
  snap: boolean
  elevation?: number
  floating?: boolean
  shadow?: boolean
}

function MovingSprite({
  name, atlas, frameId, target, timing, reducedMotion, snap, elevation = 0,
  floating = false, shadow = true,
}: MovingSpriteProps) {
  const root = useRef<Group>(null)
  const visual = useRef<Group>(null)
  const [initialPosition] = useState(() => [target.x, 0, target.z] as const)
  const motion = useRef<MotionSegment>({
    from: target, to: target, receivedAt: timing.receivedAt, duration: timing.duration, snap: true,
  })
  useLayoutEffect(() => {
    const node = root.current
    if (!node) return
    motion.current = nextMotion(
      { x: node.position.x, z: node.position.z }, target, timing, reducedMotion || snap,
    )
    if (motion.current.snap) node.position.set(target.x, 0, target.z)
  }, [reducedMotion, snap, target, timing])
  useFrame(() => {
    const node = root.current
    const visualNode = visual.current
    if (!node || !visualNode) return
    const now = performance.now()
    const point = sampleMotion(motion.current, now)
    const bob = floating && timing.running && !reducedMotion ? Math.sin(now / 260) * 0.08 : 0
    node.position.set(point.x, 0, point.z)
    visualNode.position.y = elevation + bob
  })
  return <group ref={root} name={name} position={initialPosition}>
    {shadow && <Billboard
      atlas={atlas} frameId="actor.shadow" anchor="paintedCenter" position={[0, 0.003, 0]}
      rotation={[-Math.PI / 2, 0, 0]} faceCamera={false} depthTest renderOrder={2}
      scale={1.35 / 48}
    />}
    <group ref={visual} position={[0, elevation, 0]}>
      <Billboard atlas={atlas} frameId={frameId} anchor="feet" position={[0, 0, 0]}
        scale={1.35 / 48} />
    </group>
  </group>
}

function Actors({ snapshot, atlas, timing, reducedMotion }: EntitiesProps) {
  const views = snapshot.state.actors
    .filter(({ alive }) => alive)
    .map((actor) => {
      const old = timing.previousActors.get(actor.id)
      const moving = !timing.reset && !actor.voluntaryActionsFrozen
        && old !== undefined
        && (old.position.x !== actor.position.x || old.position.y !== actor.position.y)
      return createActorView(actor, moving)
    })
  return views.map((view) => <ActorVisual
    key={`${view.identity}:${timing.resetEpoch}:${timing.actorEpochs.get(view.id) ?? 0}`}
    atlas={atlas} view={view} timing={timing} reducedMotion={reducedMotion}
  />)
}

function Mobs({ snapshot, atlas, timing, reducedMotion }: EntitiesProps) {
  const rows = snapshot.state.mobs.map((mob) => {
    const old = timing.previousMobs.get(mob.id)
    return {
      mob,
      target: mobPosition(mob),
      snap: timing.reset || (mob.frozen && old !== undefined && !old.frozen),
    }
  })
  return rows.map(({ mob, target, snap }) => <MovingSprite
    key={`${mob.id}:${timing.resetEpoch}:${timing.mobEpochs.get(mob.id) ?? 0}`}
    name={`mob:${mob.id}`} atlas={atlas} frameId={`mob.${mob.heading}`}
    target={target} timing={timing} reducedMotion={reducedMotion} snap={snap}
  />)
}

function Bombs({ snapshot, atlas, timing, reducedMotion }: EntitiesProps) {
  const animated = timing.running && !reducedMotion
  return snapshot.state.bombs.map((bomb) => {
    const view = bombView(bomb, animated)
    return <group key={bomb.id} name={`bomb:${bomb.id}`}
      position={[view.groundPosition.x, 0, view.groundPosition.z]}>
      <Billboard
        atlas={atlas} frameId="bomb.shadow" anchor="paintedCenter" position={[0, 0.015, 0]}
        rotation={[-Math.PI / 2, 0, 0]} faceCamera={false} depthTest renderOrder={2}
      />
      <Billboard atlas={atlas} frameId={view.shellFrame} position={[0, 0.38, 0]}
        scale={view.shellScale / 48} />
      {view.spark && <Billboard atlas={atlas} frameId="bomb.spark" position={[0, 0.38, 0]} renderOrder={12} />}
      {view.paused && <Billboard atlas={atlas} frameId="bomb.pause" position={[0, 0.38, 0]} renderOrder={13} />}
    </group>
  })
}

function GroundEntities({ snapshot, atlas, timing, reducedMotion }: EntitiesProps) {
  const animated = timing.running && !reducedMotion
  const burningOil = new Set(
    (snapshot.state.oilBarrels ?? [])
      .filter(({ state }) => state === 'burning')
      .map(({ cell }) => cellKey(cell)),
  )
  return <>
    {snapshot.state.pickups.map((pickup) => {
      const point = cellCenter(pickup.cell)
      return <Billboard key={cellKey(pickup.cell)} atlas={atlas} frameId={pickupFrame(pickup.kind)}
        position={[point.x, 0.5, point.z]} />
    })}
    {snapshot.state.flames.filter(({ cell }) => !burningOil.has(cellKey(cell))).map((flame) => {
      const point = cellCenter(flame.cell)
      return <Billboard key={cellKey(flame.cell)} atlas={atlas} frameId="effect.flame.0"
        position={[point.x, 0.5, point.z]}
        frameAt={(now) => `effect.flame.${animated ? Math.floor(now / 80) % 2 : 0}`} />
    })}
    {(snapshot.state.oilBarrels ?? []).map((barrel) => {
      const point = cellCenter(barrel.cell)
      const burning = barrel.state === 'burning'
      return <group key={cellKey(barrel.cell)} position={[point.x, 0, point.z]}>
        <Billboard atlas={atlas} frameId={burning ? 'oil.charred' : 'oil.unlit'} position={[0, 0.5, 0]} />
        {burning && <Billboard atlas={atlas} frameId="effect.flame.0" position={[0, 0.5, 0]}
          renderOrder={12}
          frameAt={(now) => `effect.flame.${animated ? Math.floor(now / 80) % 2 : 0}`} />}
      </group>
    })}
  </>
}

function Weather({ snapshot, atlas, timing, reducedMotion }: EntitiesProps) {
  const cloud = snapshot.state.stormCloud
  if (!cloud) return null
  return <MovingSprite
    key={`${timing.resetEpoch}:${timing.cloudEpoch}`}
    name="storm-cloud" atlas={atlas} frameId="environment.stormCloud"
    target={cloudPosition(cloud)} timing={timing} reducedMotion={reducedMotion}
    snap={timing.reset} elevation={1.15} floating
  />
}

export function Entities(props: EntitiesProps) {
  return <group>
    <GroundEntities {...props} />
    <Bombs {...props} />
    <Mobs {...props} />
    <Actors {...props} />
    <Weather {...props} />
  </group>
}
