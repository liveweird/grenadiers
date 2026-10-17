import { describe, expect, it } from 'vitest'
import { initialGameClientState, type GameClientState } from './clientTypes'
import { presentSnapshot } from './presentation'
import { snapshot } from './testSupport'

function state(): GameClientState {
  const current = snapshot(2)
  return {
    ...initialGameClientState(),
    connection: 'authenticated',
    room: current.room,
    snapshot: current,
    youSeat: 'one',
    admissionBaseline: {
      roomId: current.room.roomId, revision: 1, tick: 1, physicalTick: 1,
    },
  }
}

describe('v7 arena presentation', () => {
  it('removes hidden actors and reveals them only at their new public position', () => {
    const current = state()
    expect(presentSnapshot(current)?.state.actors.map((actor) => actor.seat)).toEqual(['one', 'two'])
    current.snapshot!.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    expect(presentSnapshot(current)?.state.actors.map((actor) => actor.seat)).toEqual(['one'])
    current.snapshot!.state.actors[1] = {
      type: 'visible', id: 2, seat: 'two', position: { x: 4096, y: 3072 }, heading: 'north',
      speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
      inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
    }
    expect(presentSnapshot(current)?.state.actors[1]).toMatchObject({
      seat: 'two', position: { x: 4096, y: 3072 }, heading: 'north',
    })
  })

  it('removes all bomb drawings during blindness while keeping fire and authoritative counts', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.liveBombCount = 2
    own.effects = [{ kind: 'invisibleBombs', remainingTicks: 120 }]
    current.snapshot!.state.bombs = [
      {
        id: 3, cell: { column: 1, row: 1 }, owner: own.id, range: 2, placedAt: 1,
        trigger: { type: 'remote' }, blast: 'standard',
      },
      {
        id: 4, cell: { column: 2, row: 1 }, owner: 2, range: 2, placedAt: 1,
        trigger: { type: 'contact' }, blast: 'piercing',
      },
    ]
    current.snapshot!.state.flames = [{ cell: { column: 3, row: 1 }, expiresAt: 30 }]

    const presented = presentSnapshot(current)!
    expect(presented.state.bombs).toEqual([])
    expect(presented.state.flames).toHaveLength(1)
    expect(own.liveBombCount).toBe(2)
  })
})
