import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import { createArenaSceneStore, DEATH_ANIMATION_MS, readSceneTiming } from './sceneStore'
import type { PresentationDeathCue } from '../types'

const death = (key: string, bornAt: number): PresentationDeathCue => ({
  key, appearance: 'one', entityId: 1, position: { x: 512, y: 512 }, heading: 'east',
  tick: 2, index: 0, bornAt,
})

describe('arena scene store', () => {
  it('publishes immutable revisions and synchronously clears the presented snapshot', () => {
    const store = createArenaSceneStore()
    const listener = vi.fn()
    const unsubscribe = store.subscribe(listener)
    const snapshot = gameSnapshot()
    const initial = store.read()
    store.update(snapshot, 125)
    expect(store.read()).toMatchObject({ snapshot, receivedAt: 125, serial: 1 })
    expect(store.read()).not.toBe(initial)
    store.clear()
    expect(store.read()).toMatchObject({ snapshot: null, receivedAt: 0, serial: 2 })
    expect(listener).toHaveBeenCalledTimes(2)
    unsubscribe()
    store.update(snapshot, 200)
    expect(listener).toHaveBeenCalledTimes(2)
  })

  it('only publishes actual reduced-motion changes', () => {
    const store = createArenaSceneStore()
    const listener = vi.fn()
    store.subscribe(listener)
    store.setReducedMotion(false)
    store.setReducedMotion(true)
    store.setReducedMotion(true)
    expect(store.read()).toMatchObject({ reducedMotion: true, serial: 1 })
    expect(listener).toHaveBeenCalledOnce()
  })

  it('copies public pose history and drops removed actors before a later reveal', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    const id = snapshot.state.actors[0].id
    const originalX = snapshot.state.actors[0].position.x
    store.update(snapshot, 0)
    const firstGeneration = readSceneTiming(store)?.actorEpochs.get(id)
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    snapshot.state.actors[0].position.x += 1024
    store.update(snapshot, 20)
    expect(readSceneTiming(store)?.previousActors.get(id)?.position.x).toBe(originalX)
    snapshot.state.actors = snapshot.state.actors.slice(1)
    store.update(snapshot, 40)
    snapshot.state.actors.push({
      id, seat: 'one', position: { x: 4096, y: 4096 }, heading: 'south', alive: true,
    })
    store.update(snapshot, 60)
    expect(readSceneTiming(store)?.previousActors.has(id)).toBe(false)
    expect(readSceneTiming(store)?.actorEpochs.get(id)).toBe((firstGeneration ?? 0) + 1)
    store.clear()
    expect(readSceneTiming(store)).toBeNull()
  })

  it('keeps clear as a monotonic reset even when the empty revision is omitted', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    store.update(snapshot, 0)
    const before = readSceneTiming(store)!.resetEpoch
    store.clear()
    store.update(snapshot, 20)
    expect(readSceneTiming(store)!.resetEpoch).toBeGreaterThan(before)
  })

  it('retains reset epochs across coalesced follow-up snapshots and preference pulses', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    store.update(snapshot, 0)
    const initialEpoch = readSceneTiming(store)!.resetEpoch
    snapshot.room.roundId += 1
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    store.update(snapshot, 20)
    const resetEpoch = readSceneTiming(store)!.resetEpoch
    expect(resetEpoch).toBe(initialEpoch + 1)
    snapshot.state.physicalTick += 1
    store.update(snapshot, 40)
    expect(readSceneTiming(store)).toMatchObject({ reset: false, resetEpoch })
    store.setReducedMotion(true)
    store.setReducedMotion(false)
    expect(readSceneTiming(store)!.resetEpoch).toBe(resetEpoch + 2)
  })

  it('persists actor, mob, and cloud generations across omitted revisions', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.mobs = [{
      id: 90, lairId: 0, position: { x: 2048, y: 2048 }, heading: 'east', frozen: false,
    }]
    snapshot.state.stormCloud = {
      position: { x: 2048, y: 2048 }, direction: 'east', legRemainingTicks: 1,
    }
    store.update(snapshot, 0)
    const first = readSceneTiming(store)!
    const actorId = snapshot.state.actors[0].id
    snapshot.state.actors = snapshot.state.actors.slice(1)
    snapshot.state.mobs = []
    snapshot.state.stormCloud = null
    store.update(snapshot, 20)
    snapshot.state.actors.push({
      id: actorId, seat: 'one', position: { x: 4096, y: 4096 }, heading: 'north', alive: true,
    })
    snapshot.state.mobs = [{
      id: 90, lairId: 0, position: { x: 4096, y: 4096 }, heading: 'west', frozen: false,
    }]
    snapshot.state.stormCloud = {
      position: { x: 4096, y: 4096 }, direction: 'west', legRemainingTicks: 1,
    }
    store.update(snapshot, 40)
    const revealed = readSceneTiming(store)!
    expect(revealed.actorEpochs.get(actorId)).toBe((first.actorEpochs.get(actorId) ?? 0) + 1)
    expect(revealed.mobEpochs.get(90)).toBe((first.mobEpochs.get(90) ?? 0) + 1)
    expect(revealed.cloudEpoch).toBe(first.cloudEpoch + 1)
  })

  it('deduplicates death envelopes, bounds lifetime, and purges on generation changes', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues: [death('first', 0)] }
    store.update(snapshot, 0)
    expect(store.read().deaths.map(({ key }) => key)).toEqual(['first'])
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    store.update(snapshot, 100)
    expect(store.read().deaths).toHaveLength(1)
    snapshot.presentationDeaths = { epoch: 1, sequence: 2, cues: [death('first', 0), death('second', 100)] }
    store.update(snapshot, 599)
    expect(store.read().deaths.map(({ key }) => key)).toEqual(['first', 'second'])
    store.update(snapshot, 600)
    expect(store.read().deaths.map(({ key }) => key)).toEqual(['second'])
    store.clear()
    store.update(snapshot, 610)
    expect(store.read().deaths).toEqual([])
    snapshot.presentationDeaths = { epoch: 2, sequence: 3, cues: [] }
    store.update(snapshot, 610)
    expect(store.read().deaths).toEqual([])
  })

  it.each([true, false])('purges and fences active deaths when reduced motion becomes %s', (reduced) => {
    const store = createArenaSceneStore()
    store.setReducedMotion(!reduced)
    const snapshot = gameSnapshot()
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues: [death('reduced', 0)] }
    store.update(snapshot, 0)
    expect(store.read().deaths).toHaveLength(1)
    store.setReducedMotion(reduced)
    expect(store.read().deaths).toEqual([])
    snapshot.presentationDeaths.sequence = 2
    store.update(snapshot, 100)
    expect(store.read().deaths).toEqual([])
    store.setReducedMotion(!reduced)
    store.update(snapshot, 101)
    expect(store.read().deaths).toEqual([])
    vi.restoreAllMocks()
  })

  it('retains blood pools after body expiry and across reduced-motion changes', () => {
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.presentationDeaths = { epoch: 1, sequence: 0, cues: [] }
    store.update(snapshot, 0)
    snapshot.presentationDeaths = { epoch: 1, sequence: 1, cues: [death('pool', 0)] }
    store.update(snapshot, 0)
    expect(store.read()).toMatchObject({ deaths: [{ key: 'pool' }], bloodPools: [{ key: 'blood:0:0' }] })
    store.update(snapshot, DEATH_ANIMATION_MS)
    expect(store.read().deaths).toEqual([])
    expect(store.read().bloodPools).toHaveLength(1)
    store.setReducedMotion(true)
    expect(store.read().bloodPools).toHaveLength(1)
  })
})
