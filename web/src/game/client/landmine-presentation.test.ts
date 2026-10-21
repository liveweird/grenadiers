import { expect, it } from 'vitest'
import { snapshot } from './testSupport'
import { presentSnapshot } from './presentation'
import { initialGameClientState } from './clientTypes'

it('shows the stored Landmine charge without inferring a global armed state', () => {
  const wire = snapshot(2)
  const own = wire.state.actors[0]
  if (own.type !== 'visible') throw new Error('expected visible actor')
  own.inventory.weapon = 'landmine'
  wire.state.bombs = [{
    id: 3, cell: { column: 3, row: 1 }, owner: own.id, range: 2, placedAt: 1,
    trigger: { type: 'contact' }, blast: 'standard',
  }]
  own.liveBombCount = 1
  const presented = presentSnapshot({
    ...initialGameClientState(), connection: 'authenticated', room: wire.room, snapshot: wire,
    youSeat: 'one', admissionBaseline: null,
  })
  expect(presented?.state.actors[0].landmineStored).toBe(true)
  expect(presented?.state.actors[0].remoteArmed).toBe(false)
  expect(presented?.state.bombs[0].trigger).toEqual({ type: 'contact' })
})
