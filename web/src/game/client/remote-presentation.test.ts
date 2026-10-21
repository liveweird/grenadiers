import { expect, it } from 'vitest'
import { initialGameClientState } from './clientTypes'
import { presentSnapshot } from './presentation'
import { snapshot } from './testSupport'

it('derives the armed Remote cue from the owned world bomb independently of stored weapon', () => {
  const wire = snapshot(2)
  const own = wire.state.actors[0]
  if (own.type !== 'visible') throw new Error('expected visible own actor')
  own.inventory.weapon = 'powerBomb'
  wire.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'remoteBomb' }]
  wire.state.bombs = [{
    id: 3, cell: { column: 3, row: 1 }, owner: own.id, range: 2, placedAt: 1,
    trigger: { type: 'remote' }, blast: 'standard',
  }]
  const presented = presentSnapshot({
    ...initialGameClientState(), connection: 'authenticated', room: wire.room, snapshot: wire,
    youSeat: 'one', admissionBaseline: {
      roomId: wire.room.roomId, revision: 1, tick: 1, physicalTick: 1,
    },
  })!
  expect(presented.state.actors[0]).toMatchObject({ remoteArmed: true, powerArmed: true })
  expect(presented.state.actors[1].remoteArmed).toBe(false)
  expect(presented.state.pickups).toEqual(wire.state.pickups)
  expect(presented.state.bombs[0].trigger).toEqual({ type: 'remote' })
})
