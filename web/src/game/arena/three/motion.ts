import type { SnapshotMessage } from '../types'

export interface GroundPosition { x: number; z: number }
interface ActorMotionRow {
  position: { x: number; y: number }
  frozen: boolean
  teleportMarker: number | null | undefined
}
interface MobMotionRow { frozen: boolean }
export interface MotionTiming {
  identity: string
  tick: number
  receivedAt: number
  duration: number
  reset: boolean
  running: boolean
  actors: ReadonlyMap<number, ActorMotionRow>
  previousActors: ReadonlyMap<number, ActorMotionRow>
  mobs: ReadonlyMap<number, MobMotionRow>
  previousMobs: ReadonlyMap<number, MobMotionRow>
  actorEpochs: ReadonlyMap<number, number>
  mobEpochs: ReadonlyMap<number, number>
  cloudEpoch: number
  cloudPresent: boolean
  resetEpoch: number
}
export interface MotionSegment {
  from: GroundPosition
  to: GroundPosition
  receivedAt: number
  duration: number
  snap: boolean
}

const FRAME_MS = 1000 / 60
export const unitsToGround = (position: { x: number; y: number }): GroundPosition => ({
  x: position.x / 1024,
  z: position.y / 1024,
})
const sceneIdentity = (snapshot: SnapshotMessage): string => [
  snapshot.room.roomId,
  snapshot.room.roundId,
  snapshot.room.presentationBaseline ?? 'legacy',
].join(':')
const snapshotTick = (snapshot: SnapshotMessage): number =>
  snapshot.state.physicalTick ?? snapshot.state.runningUpdates ?? 0

const nextActorRows = (
  snapshot: SnapshotMessage, previous: MotionTiming | null,
): { rows: Map<number, ActorMotionRow>; epochs: Map<number, number> } => {
  const epochs = new Map(previous?.actorEpochs)
  const rows = new Map<number, ActorMotionRow>()
  for (const actor of snapshot.state.actors) {
    if (!actor.alive) continue
    const old = previous?.actors.get(actor.id)
    const snapEdge = !old
      || old.teleportMarker !== actor.lastTeleportTick
      || (Boolean(actor.voluntaryActionsFrozen) && !old.frozen)
    if (snapEdge) epochs.set(actor.id, (epochs.get(actor.id) ?? 0) + 1)
    rows.set(actor.id, {
      position: { ...actor.position },
      frozen: Boolean(actor.voluntaryActionsFrozen),
      teleportMarker: actor.lastTeleportTick,
    })
  }
  return { rows, epochs }
}

const nextMobRows = (
  snapshot: SnapshotMessage, previous: MotionTiming | null,
): { rows: Map<number, MobMotionRow>; epochs: Map<number, number> } => {
  const epochs = new Map(previous?.mobEpochs)
  const rows = new Map<number, MobMotionRow>()
  for (const mob of snapshot.state.mobs) {
    const old = previous?.mobs.get(mob.id)
    if (!old || (mob.frozen && !old.frozen)) {
      epochs.set(mob.id, (epochs.get(mob.id) ?? 0) + 1)
    }
    rows.set(mob.id, { frozen: mob.frozen })
  }
  return { rows, epochs }
}

const nextResetEpoch = (
  previous: MotionTiming | null, reset: boolean, running: boolean,
): number => {
  const enteredTerminal = Boolean(previous?.running && !running)
  return (previous?.resetEpoch ?? 0) + (reset || enteredTerminal ? 1 : 0)
}

export function motionTiming(
  snapshot: SnapshotMessage,
  receivedAt: number,
  previous: MotionTiming | null,
): MotionTiming {
  const identity = sceneIdentity(snapshot)
  const tick = snapshotTick(snapshot)
  const tickDelta = previous ? tick - previous.tick : 0
  const reset = !previous || identity !== previous.identity || tickDelta < 0
  const actors = nextActorRows(snapshot, previous)
  const mobs = nextMobRows(snapshot, previous)
  const running = snapshot.room.phase.type === 'running'
  const cloudPresent = Boolean(snapshot.state.stormCloud)
  const cloudEpoch = (previous?.cloudEpoch ?? 0)
    + (cloudPresent && !previous?.cloudPresent ? 1 : 0)
  return {
    identity,
    tick,
    receivedAt,
    duration: Math.max(FRAME_MS, Math.min(100, tickDelta * FRAME_MS)),
    reset,
    running,
    actors: actors.rows,
    previousActors: reset ? new Map() : previous?.actors ?? new Map(),
    mobs: mobs.rows,
    previousMobs: reset ? new Map() : previous?.mobs ?? new Map(),
    actorEpochs: actors.epochs,
    mobEpochs: mobs.epochs,
    cloudEpoch,
    cloudPresent,
    resetEpoch: nextResetEpoch(previous, reset, running),
  }
}

/** Persists preference-driven snap semantics even if React coalesces the revision. */
export const advanceResetEpoch = (timing: MotionTiming): MotionTiming => ({
  ...timing,
  reset: true,
  resetEpoch: timing.resetEpoch + 1,
})

export const sampleMotion = (segment: MotionSegment, now: number): GroundPosition => {
  const progress = segment.snap ? 1
    : Math.max(0, Math.min(1, (now - segment.receivedAt) / segment.duration))
  return {
    x: segment.from.x + (segment.to.x - segment.from.x) * progress,
    z: segment.from.z + (segment.to.z - segment.from.z) * progress,
  }
}

export function nextMotion(
  displayed: GroundPosition,
  target: GroundPosition,
  timing: MotionTiming,
  snap: boolean,
): MotionSegment {
  const shouldSnap = snap || timing.reset || !timing.running
  return {
    from: shouldSnap ? target : displayed,
    to: target,
    receivedAt: timing.receivedAt,
    duration: timing.duration,
    snap: shouldSnap,
  }
}

