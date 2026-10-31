import Ajv from 'ajv'
import { describe, expect, it } from 'vitest'
import v16Schema from '../../../../api/gameplay-v16.schema.json'
import v17Schema from '../../../../api/gameplay-v17.schema.json'
import { snapshot } from '../client/testSupport'
import type { BombView, Snapshot, VisibleActorView } from './schema'
import { validateGameplayV17ServerMessage } from './validation'

function own(frame: Snapshot): VisibleActorView {
  const actor = frame.state.actors[0]
  if (actor?.type !== 'visible') throw new Error('expected visible owner')
  return actor
}

function bomb(id: number, owner: number, index: number, contact: boolean): BombView {
  return {
    id, owner, cell: { column: index % 15, row: Math.floor(index / 15) },
    range: 1, placedAt: 1, blast: 'standard',
    trigger: contact ? { type: 'contact' } : { type: 'timed', remainingTicks: 60, paused: false },
  }
}

function fullRegularSlotWithMines(mineCount = 33): Snapshot {
  const frame = structuredClone(snapshot(2))
  const owner = own(frame)
  owner.bombCapacity = 1
  owner.liveBombCount = 1
  frame.state.bombs = [bomb(100, owner.id, 0, false)]
  for (let index = 0; index < mineCount; index += 1) {
    frame.state.bombs.push(bomb(101 + index, owner.id, index + 1, true))
  }
  return frame
}

describe('gameplay v17 mine accounting', () => {
  it('accepts a full ordinary slot plus 33 extra contact mines', () => {
    const frame = fullRegularSlotWithMines()
    expect(validateGameplayV17ServerMessage(frame)).toEqual({ valid: true, value: frame })

    const v16 = new Ajv({ allErrors: true }).addSchema(v16Schema)
      .getSchema(`${v16Schema.$id}#/$defs/GameState`)!
    const v17 = new Ajv({ allErrors: true }).addSchema(v17Schema)
      .getSchema(`${v17Schema.$id}#/$defs/GameState`)!
    expect(v17(frame.state)).toBe(true)
    expect(v16(frame.state)).toBe(false)
    expect(v16.errors).toEqual(expect.arrayContaining([
      expect.objectContaining({ instancePath: '/bombs', keyword: 'maxItems' }),
    ]))
  })

  it('excludes contact mines from liveBombCount and rejects ordinary mismatches', () => {
    const frame = fullRegularSlotWithMines(1)
    own(frame).bombCapacity = 2
    own(frame).liveBombCount = 2
    expect(validateGameplayV17ServerMessage(frame)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidWorldState',
    })

    const onlyMine = fullRegularSlotWithMines(1)
    onlyMine.state.bombs.shift()
    own(onlyMine).liveBombCount = 0
    expect(validateGameplayV17ServerMessage(onlyMine)).toEqual({ valid: true, value: onlyMine })
  })

  it('does not refund or increase ordinary count when a mine disappears', () => {
    const frame = fullRegularSlotWithMines(1)
    frame.state.bombs.pop()
    expect(validateGameplayV17ServerMessage(frame)).toEqual({ valid: true, value: frame })
    own(frame).liveBombCount += 1
    expect(validateGameplayV17ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it('rejects multiple bombs occupying one tile', () => {
    const frame = fullRegularSlotWithMines(1)
    frame.state.bombs[1]!.cell = { ...frame.state.bombs[0]!.cell }
    expect(validateGameplayV17ServerMessage(frame)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidWorldState',
    })
  })

  it('does not infer any actor bomb ownership when visible Invisible bombs hides the batch', () => {
    const frame = fullRegularSlotWithMines(0)
    const owner = own(frame)
    owner.effects.push({ kind: 'invisibleBombs', remainingTicks: 60 })
    frame.state.bombs = []
    const other = frame.state.actors[1]
    if (other?.type !== 'visible') throw new Error('expected visible opponent')
    other.liveBombCount = 1
    expect(validateGameplayV17ServerMessage(frame)).toEqual({ valid: true, value: frame })
  })

  it('still enforces the visible regular lower bound and ordinary capacity under incomplete visibility', () => {
    const frame = fullRegularSlotWithMines(0)
    own(frame).effects.push({ kind: 'invisibleBombs', remainingTicks: 60 })
    own(frame).liveBombCount = 0
    expect(validateGameplayV17ServerMessage(frame)).toMatchObject({ valid: false })

    own(frame).liveBombCount = 2
    expect(validateGameplayV17ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it('widens only v17 snapshot event batches to 256 with indices capped at 215', () => {
    const frame = fullRegularSlotWithMines(0)
    frame.state.bombs = []
    own(frame).liveBombCount = 0
    frame.events = Array.from({ length: 256 }, (_, index) => ({
      tick: frame.state.physicalTick, index: index % 216,
      kind: { type: 'roundEnded' as const,
        result: { type: 'draw' as const, reason: 'timeout' as const } },
    }))
    const v16 = new Ajv({ allErrors: true }).addSchema(v16Schema)
      .getSchema(`${v16Schema.$id}#/$defs/Snapshot`)!
    const v17 = new Ajv({ allErrors: true }).addSchema(v17Schema)
      .getSchema(`${v17Schema.$id}#/$defs/Snapshot`)!
    expect(v17(frame)).toBe(true)
    expect(v16(frame)).toBe(false)
    frame.events[0]!.index = 216
    expect(v17(frame)).toBe(false)
    frame.events[0]!.index = 0
    frame.events.push(frame.events[0]!)
    expect(v17(frame)).toBe(false)
  })
})
