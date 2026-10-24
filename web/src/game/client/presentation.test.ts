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

describe('v11 arena presentation', () => {
  it('removes hidden actors and reveals them only at their new public position', () => {
    const current = state()
    expect(presentSnapshot(current)?.state.actors.map((actor) => actor.seat)).toEqual(['one', 'two'])
    current.snapshot!.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    expect(presentSnapshot(current)?.state.actors.map((actor) => actor.seat)).toEqual(['one'])
    current.snapshot!.state.actors[1] = {
      type: 'visible', id: 2, seat: 'two', position: { x: 4096, y: 3072 }, heading: 'north',
      speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
      inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
      voluntaryActionsFrozen: false, lastTeleportTick: null,
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
    expect(presented.state.actors[0].invisibleBombsRemainingTicks).toBe(120)
    expect(own.liveBombCount).toBe(2)
    current.snapshot!.state.pickups = [{ cell: { column: 4, row: 1 }, kind: 'invisibleBombs' }]
    expect(presentSnapshot(current)?.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('presents Shield pickups and current protection from the authoritative effect clock', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [{ kind: 'shield', remainingTicks: 299 }]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shield' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0].shieldRemainingTicks).toBe(299)
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('carries the teleport epoch and renders the distinct pickup kind', () => {
    const current = state()
    for (const actor of current.snapshot!.state.actors) {
      if (actor.type === 'visible') actor.lastTeleportTick = 2
    }
    current.snapshot!.state.pickups = [{ cell: { column: 11, row: 11 }, kind: 'teleporter' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors.map(({ lastTeleportTick }) => lastTeleportTick)).toEqual([2, 2])
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('presents only current Wall Pass equipment and never invents private exit-grace state', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.inventory.equipment = 'wallPass'
    current.snapshot!.state.pickups = [{ cell: { column: 3, row: 2 }, kind: 'wallPass' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({ wallPassEquipped: true })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
    expect(Object.keys(presented.state.actors[0])).not.toContain('wallPassExitGrace')

    own.inventory.equipment = 'heart'
    expect(presentSnapshot(current)!.state.actors[0]).toMatchObject({ wallPassEquipped: false })
  })

  it('presents Kick equipment and pickup without inventing private bomb motion', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.inventory.equipment = 'kick'
    current.snapshot!.state.pickups = [{ cell: { column: 3, row: 2 }, kind: 'kick' }]
    current.snapshot!.state.bombs = [{
      id: 9, cell: { column: 4, row: 2 }, owner: own.id, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 100, paused: false }, blast: 'standard',
    }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({ kickEquipped: true })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
    expect(presented.state.bombs[0]).toEqual(current.snapshot!.state.bombs[0])
    expect(Object.keys(presented.state.bombs[0])).toEqual([
      'id', 'cell', 'owner', 'range', 'placedAt', 'trigger', 'blast',
    ])
  })

  it('presents Punch equipment and its distinct pickup without private cooldown or bomb history', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.inventory.equipment = 'punch'
    current.snapshot!.state.pickups = [{ cell: { column: 3, row: 2 }, kind: 'punch' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({ punchEquipped: true })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
    expect(Object.keys(presented.state.actors[0])).not.toContain('lastSuccessfulPunchTick')
  })

  it('presents Bracelet statically while shove movement and hidden actors stay authoritative', () => {
    const current = state()
    const [holder, victim] = current.snapshot!.state.actors
    if (holder.type !== 'visible' || victim.type !== 'visible') throw new Error('visible actors expected')
    holder.inventory.equipment = 'bracelet'
    victim.position = { x: 12_800, y: 12_800 }
    current.snapshot!.state.pickups = [{ cell: { column: 3, row: 2 }, kind: 'bracelet' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({ braceletEquipped: true })
    expect(presented.state.actors[1]).toMatchObject({ position: { x: 12_800, y: 12_800 } })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
    expect(Object.keys(presented.state.actors[0])).not.toContain('lastSuccessfulShoveTick')

    current.snapshot!.state.actors[1] = { type: 'hidden', id: victim.id, seat: victim.seat, alive: true }
    expect(presentSnapshot(current)!.state.actors.map(({ id }) => id)).toEqual([holder.id])
  })

  it('presents only materialized Oil objects without inventory, timers, or pending facts', () => {
    const current = state()
    current.snapshot!.room.settings.enabledItems = ['oilBarrel']
    current.snapshot!.state.oilBarrels = [
      { cell: { column: 3, row: 1 }, state: 'unlit' },
      { cell: { column: 5, row: 1 }, state: 'burning' },
    ]
    const presented = presentSnapshot(current)!
    expect(presented.state.oilBarrels).toEqual(current.snapshot!.state.oilBarrels)
    expect(presented.state.pickups).toEqual([])
    expect(Object.keys(presented.state)).not.toContain('pendingOilSites')
    expect(Object.keys(presented.state.actors[0])).not.toContain('oilBarrel')
  })

})

describe('Stopwatch presentation', () => {
  it('presents its holder, frozen status, and an authoritative paused Timed bomb', () => {
    const current = state()
    const [holder, victim] = current.snapshot!.state.actors
    if (holder.type !== 'visible' || victim.type !== 'visible') throw new Error('visible actors expected')
    holder.effects = [{ kind: 'stopwatch', remainingTicks: 179 }]
    victim.voluntaryActionsFrozen = true
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'stopwatch' }]
    current.snapshot!.state.bombs = [{
      id: 9, cell: { column: 4, row: 4 }, owner: holder.id, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 120, paused: true }, blast: 'standard',
    }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({ stopwatchRemainingTicks: 179 })
    expect(presented.state.actors[1]).toMatchObject({ voluntaryActionsFrozen: true })
    expect(presented.state.bombs[0].trigger).toEqual({ type: 'timed', remainingTicks: 120, paused: true })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })
})

describe('v11 timed item presentation', () => {
  it('presents Golden Flame pickups and its reconnect-safe authoritative clock', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [
      { kind: 'shield', remainingTicks: 120 },
      { kind: 'goldenFlame', remainingTicks: 599 },
    ]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'goldenFlame' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({
      shieldRemainingTicks: 120, goldenFlameRemainingTicks: 599,
    })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('presents Short Fuse pickups and its current authoritative clock alongside other effects', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [
      { kind: 'shield', remainingTicks: 120 },
      { kind: 'goldenFlame', remainingTicks: 480 },
      { kind: 'shortFuse', remainingTicks: 599 },
    ]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFuse' }]
    current.admissionBaseline = {
      roomId: current.room!.roomId, revision: 2, tick: 2, physicalTick: 2,
    }
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({
      shieldRemainingTicks: 120,
      goldenFlameRemainingTicks: 480,
      shortFuseRemainingTicks: 599,
    })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
    expect(presented.room.presentationBaseline).toContain(':2:2:2')
  })

  it('presents only current Short Flame state on a quiet reconnect snapshot', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [
      { kind: 'goldenFlame', remainingTicks: 480 },
      { kind: 'shortFlame', remainingTicks: 600 },
    ]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'shortFlame' }]
    current.snapshot!.events = []
    current.admissionBaseline = {
      roomId: current.room!.roomId, revision: 9, tick: 42, physicalTick: 42,
    }
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({
      goldenFlameRemainingTicks: 480, shortFlameRemainingTicks: 600,
    })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
    expect(presented.room.presentationBaseline).toContain(':9:42:42')
  })

  it('does not turn historical collection events into a Short Flame cue', () => {
    const current = state()
    current.snapshot!.events = [{
      tick: 1, index: 0, kind: {
        type: 'pickupCollected', id: 1, seat: 'one',
        cell: { column: 2, row: 2 }, kind: 'shortFlame',
      },
    }]
    expect(presentSnapshot(current)?.state.actors[0].shortFlameRemainingTicks).toBe(0)
  })

  it('presents Long Fuse pickup and quiet authoritative timer independently', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [
      { kind: 'shortFuse', remainingTicks: 120 },
      { kind: 'longFuse', remainingTicks: 600 },
    ]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'longFuse' }]
    current.snapshot!.events = []
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({
      shortFuseRemainingTicks: 120, longFuseRemainingTicks: 600,
    })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('presents Reverse Controls without changing the authoritative heading', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.heading = 'east'
    own.effects = [{ kind: 'reverseControls', remainingTicks: 600 }]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'reverseControls' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0]).toMatchObject({
      heading: 'east', reverseControlsRemainingTicks: 600,
    })
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('presents Diarrhea without synthesizing client bomb commands', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [{ kind: 'diarrhea', remainingTicks: 599 }]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'diarrhea' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0].diarrheaRemainingTicks).toBe(599)
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })

  it('preserves every Slow-foot stack timer for grouped presentation', () => {
    const current = state()
    const own = current.snapshot!.state.actors[0]
    if (own.type !== 'visible') throw new Error('expected visible own actor')
    own.effects = [
      { kind: 'slowFoot', remainingTicks: 600 },
      { kind: 'slowFoot', remainingTicks: 431 },
      { kind: 'slowFoot', remainingTicks: 12 },
    ]
    current.snapshot!.state.pickups = [{ cell: { column: 2, row: 2 }, kind: 'slowFoot' }]
    const presented = presentSnapshot(current)!
    expect(presented.state.actors[0].slowFootRemainingTicks).toEqual([600, 431, 12])
    expect(presented.state.pickups).toEqual(current.snapshot!.state.pickups)
  })
})
