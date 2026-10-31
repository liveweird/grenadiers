import { describe, expect, it } from 'vitest'
import { projectCpuDebug } from './cpuDebugProjection'
import type { CpuDebugResponse } from './cpuDebugTypes'
import type { GameClientState } from './clientTypes'
import { snapshot } from './testSupport'

function debugState(): GameClientState {
  const current = snapshot(20)
  current.room.configuration.controllers[1] = 'cpu'
  current.room.configuration.cpuDifficulty = 'standard'
  current.room.seats[1].controller = { type: 'cpu', difficulty: 'standard' }
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: {
      roomId: current.room.roomId, revision: current.room.revision,
      tick: current.room.tick, physicalTick: current.state.physicalTick,
    },
  }
}

function response(target: Extract<CpuDebugResponse['cpus'][number], { status: 'available' }>['target'] = null): CpuDebugResponse {
  return {
    protocolVersion: 1, roomId: 'b'.repeat(32), roundId: 1, tick: 20, revision: 20, youSeat: 'one',
    cpus: [{
      seat: 'two', status: 'available', observedTick: 20, objective: 'wait', target,
      approach: null, phase: 'waiting', switchReason: 'noTargets',
    }],
  }
}

describe('CPU debug display projection', () => {
  it('keeps a visible CPU intent available without exposing actor coordinates', () => {
    const [row] = projectCpuDebug(response(), debugState())
    expect(row.intent).toMatchObject({ seat: 'two', objective: 'wait' })
    expect(JSON.stringify(row)).not.toContain('position')
  })

  it('replaces the entire row when the CPU becomes hidden', () => {
    const state = debugState()
    state.snapshot!.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    expect(projectCpuDebug(response(), state)).toEqual([{ seat: 'two', intent: null }])
  })

  it('replaces the entire row when a pickup target is absent from the latest public snapshot', () => {
    const target = { type: 'pickup' as const, cell: { column: 3, row: 4 }, kind: 'heart' as const }
    expect(projectCpuDebug(response(target), debugState())).toEqual([{ seat: 'two', intent: null }])
  })

  it('replaces the entire row when an opponent target is hidden', () => {
    const state = debugState()
    state.snapshot!.state.actors[0] = { type: 'hidden', id: 1, seat: 'one', alive: true }
    expect(projectCpuDebug(response({ type: 'opponent', seat: 'one' }), state))
      .toEqual([{ seat: 'two', intent: null }])
  })

  it('replaces every row while the recipient is bomb-blind', () => {
    const state = debugState()
    const own = state.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects.push({ kind: 'invisibleBombs', remainingTicks: 60 })
    expect(projectCpuDebug(response(), state)).toEqual([{ seat: 'two', intent: null }])
  })
})
