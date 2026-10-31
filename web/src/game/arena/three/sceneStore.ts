import type { ArenaSceneStore, SceneRevision } from './contracts'
import type { MotionTiming } from './motion'
import { advanceResetEpoch, motionTiming } from './motion'
import type { PresentationDeathCue, SnapshotMessage } from '../types'

export const DEATH_FRAME_MS = 150
export const DEATH_ANIMATION_MS = DEATH_FRAME_MS * 4
export const REDUCED_DEATH_MS = 180
const MAX_SCENE_DEATHS = 64
const MAX_SEEN_DEATHS = 128

const timings = new WeakMap<ArenaSceneStore, { current: MotionTiming | null }>()

/** Timing is advanced by store updates so React render does not mutate presentation history. */
export const readSceneTiming = (store: ArenaSceneStore): MotionTiming | null =>
  timings.get(store)?.current ?? null

const initialRevision = (): SceneRevision => ({
  snapshot: null,
  deaths: [],
  reducedMotion: false,
  receivedAt: 0,
  serial: 0,
})

/** Small external store: snapshots rerender React, animation remains in the frame loop. */
export function createArenaSceneStore(): ArenaSceneStore {
  let revision = initialRevision()
  let resetEpochFloor = 0
  let deathEpoch: number | null = null
  let deathSequence = -1
  let deathIdentity = ''
  let seenDeaths = new Set<string>()
  const listeners = new Set<() => void>()
  const timing = { current: null as MotionTiming | null }
  const publish = (next: SceneRevision): void => {
    revision = next
    for (const listener of listeners) listener()
  }
  const resetDeaths = (): void => {
    deathEpoch = null
    deathSequence = -1
    deathIdentity = ''
    seenDeaths = new Set()
  }
  const updateDeaths = (snapshot: SnapshotMessage, now: number): readonly PresentationDeathCue[] => {
    const presentation = snapshot.presentationDeaths
    if (!presentation) {
      resetDeaths()
      return []
    }
    const identity = `${snapshot.room.roomId}:${snapshot.room.roundId}:${snapshot.room.presentationBaseline ?? 'legacy'}`
    if (deathEpoch !== presentation.epoch || deathIdentity !== identity
      || presentation.sequence < deathSequence) {
      deathEpoch = presentation.epoch
      deathIdentity = identity
      deathSequence = -1
      seenDeaths = new Set()
      revision = { ...revision, deaths: [] }
    }
    const lifetime = revision.reducedMotion ? REDUCED_DEATH_MS : DEATH_ANIMATION_MS
    const alive = revision.deaths.filter(({ bornAt }) => now - bornAt < lifetime)
    if (presentation.sequence !== deathSequence) {
      for (const cue of presentation.cues) {
        if (seenDeaths.has(cue.key)) continue
        seenDeaths.add(cue.key)
        if (seenDeaths.size > MAX_SEEN_DEATHS) {
          const oldest = seenDeaths.values().next().value as string | undefined
          if (oldest) seenDeaths.delete(oldest)
        }
        if (now - cue.bornAt < lifetime) alive.push(cue)
      }
      deathSequence = presentation.sequence
    }
    return alive.slice(-MAX_SCENE_DEATHS)
  }

  const store: ArenaSceneStore = {
    read: () => revision,
    subscribe(listener) {
      listeners.add(listener)
      return () => listeners.delete(listener)
    },
    update(snapshot, receivedAt) {
      const nextTiming = motionTiming(snapshot, receivedAt, timing.current)
      timing.current = nextTiming.resetEpoch < resetEpochFloor
        ? { ...nextTiming, resetEpoch: resetEpochFloor }
        : nextTiming
      resetEpochFloor = timing.current.resetEpoch
      const deaths = updateDeaths(snapshot, receivedAt)
      publish({ ...revision, snapshot, deaths, receivedAt, serial: revision.serial + 1 })
    },
    clear() {
      resetEpochFloor += 1
      timing.current = null
      publish({ ...revision, snapshot: null, deaths: [], receivedAt: 0, serial: revision.serial + 1 })
    },
    setReducedMotion(reducedMotion) {
      if (revision.reducedMotion === reducedMotion) return
      if (timing.current) timing.current = advanceResetEpoch(timing.current)
      resetEpochFloor = timing.current?.resetEpoch ?? resetEpochFloor + 1
      publish({ ...revision, deaths: [], reducedMotion, serial: revision.serial + 1 })
    },
  }
  timings.set(store, timing)
  return store
}
