import { act, renderHook } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { GameClientState } from './clientTypes'
import { snapshot } from './testSupport'
import { useGameClient } from './useGameClient'

const hook = vi.hoisted(() => ({
  publish: null as ((state: GameClientState) => void) | null,
  dispose: vi.fn(),
}))

vi.mock('./client', async (importOriginal) => {
  const actual = await importOriginal<typeof import('./client')>()
  return {
    ...actual,
    GameClient: class {
      constructor(publish: (state: GameClientState) => void) { hook.publish = publish }
      dispose = hook.dispose
    },
  }
})

const state = (revision: number): GameClientState => {
  const wire = snapshot(revision)
  return {
    connection: 'authenticated', room: wire.room, snapshot: wire, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: { roomId: wire.room.roomId, revision: 1, tick: 1, physicalTick: 1 },
  }
}

afterEach(() => {
  hook.publish = null
  hook.dispose.mockClear()
  vi.restoreAllMocks()
})

describe('useGameClient death latch', () => {
  it('consumes event-bearing callbacks synchronously before React coalesces state', () => {
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const result = renderHook(() => useGameClient())
    act(() => hook.publish?.(state(1)))
    const killed = state(2)
    const actor = killed.snapshot!.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.alive = false
    killed.snapshot!.events = [{
      tick: 2, index: 0, kind: { type: 'actorEliminated', id: actor.id, seat: actor.seat },
    }]
    act(() => {
      now = 20
      hook.publish?.(killed)
      now = 40
      hook.publish?.(state(3))
    })
    expect(result.result.current[0].room?.revision).toBe(3)
    expect(result.result.current[2].cues).toHaveLength(1)
    expect(result.result.current[2].cues[0]).toMatchObject({ entityId: actor.id, bornAt: 20 })
    result.unmount()
    expect(hook.dispose).toHaveBeenCalledOnce()
  })
})
