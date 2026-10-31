import type { ArenaSceneStore, SceneRevision } from './contracts'
import type { MotionTiming } from './motion'
import { advanceResetEpoch, motionTiming } from './motion'

const timings = new WeakMap<ArenaSceneStore, { current: MotionTiming | null }>()

/** Timing is advanced by store updates so React render does not mutate presentation history. */
export const readSceneTiming = (store: ArenaSceneStore): MotionTiming | null =>
  timings.get(store)?.current ?? null

const initialRevision = (): SceneRevision => ({
  snapshot: null,
  reducedMotion: false,
  receivedAt: 0,
  serial: 0,
})

/** Small external store: snapshots rerender React, animation remains in the frame loop. */
export function createArenaSceneStore(): ArenaSceneStore {
  let revision = initialRevision()
  let resetEpochFloor = 0
  const listeners = new Set<() => void>()
  const timing = { current: null as MotionTiming | null }
  const publish = (next: SceneRevision): void => {
    revision = next
    for (const listener of listeners) listener()
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
      publish({ ...revision, snapshot, receivedAt, serial: revision.serial + 1 })
    },
    clear() {
      resetEpochFloor += 1
      timing.current = null
      publish({ ...revision, snapshot: null, receivedAt: 0, serial: revision.serial + 1 })
    },
    setReducedMotion(reducedMotion) {
      if (revision.reducedMotion === reducedMotion) return
      if (timing.current) timing.current = advanceResetEpoch(timing.current)
      resetEpochFloor = timing.current?.resetEpoch ?? resetEpochFloor + 1
      publish({ ...revision, reducedMotion, serial: revision.serial + 1 })
    },
  }
  timings.set(store, timing)
  return store
}
