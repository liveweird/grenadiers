import { describe, expect, it } from 'vitest'
import { checkedCatalogMaskRanges, executeCatalogMaskRange } from '../../test/catalogMasks'
import { admitted, snapshot } from '../client/testSupport'
import { validateGameplayV10ServerMessage } from './validation'

const supported = [
  'kick', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
] as const
const supportedRanges = checkedCatalogMaskRanges({
  protocol: 'gameplay v10', catalog: supported, expectedCount: 19, expectedOrder: supported,
})

function withMarker(marker: number | null) {
  const message = snapshot(2)
  message.state.physicalTick = 2
  message.state.runningUpdates = 2
  for (const actor of message.state.actors) {
    if (actor.type === 'visible') actor.lastTeleportTick = marker
  }
  return message
}

describe('gameplay v10 item validation', () => {
  it('pins nineteen unique settings entries before exhaustively enumerating all masks', () => {
    expect(supported).toHaveLength(19)
    expect(new Set(supported)).toHaveLength(19)
    expect(2 ** supported.length).toBe(524_288)
    expect(supportedRanges).toHaveLength(512)
  })

  it('accepts enabled Wall Pass equipment, pickups, and collection history', () => {
    const message = withMarker(null)
    message.room.settings.enabledItems = ['wallPass']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.inventory.equipment = 'wallPass'
    message.state.pickups = [{ cell: { column: 3, row: 2 }, kind: 'wallPass' }]
    message.events = [{ tick: 2, index: 0, kind: {
      type: 'pickupCollected', id: actor.id, seat: actor.seat,
      cell: { column: 2, row: 2 }, kind: 'wallPass',
    } }]
    expect(validateGameplayV10ServerMessage(message)).toEqual({ valid: true, value: message })

    message.room.settings.enabledItems = []
    expect(validateGameplayV10ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('accepts public Kick facts only when enabled and rejects private motion history', () => {
    const message = withMarker(null)
    message.room.settings.enabledItems = ['kick']
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.inventory.equipment = 'kick'
    message.state.pickups = [{ cell: { column: 3, row: 2 }, kind: 'kick' }]
    expect(validateGameplayV10ServerMessage(message)).toEqual({ valid: true, value: message })

    message.room.settings.enabledItems = []
    expect(validateGameplayV10ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
    const leaked = structuredClone(message) as unknown as Record<string, unknown>
    const bombs = (leaked.state as { bombs: Array<Record<string, unknown>> }).bombs
    bombs.push({ id: 3, cell: { column: 2, row: 2 }, owner: 1, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 10, paused: false }, blast: 'standard',
      motion: { direction: 'east', countdown: 9 }, hasBeenKicked: true })
    expect(validateGameplayV10ServerMessage(leaked)).toMatchObject({ valid: false, source: 'schema' })
  })

  it('accepts the current Wall Pass catalog in fresh admission and lobby messages', () => {
    const admission = admitted()
    admission.room.settings.enabledItems = ['wallPass']
    const actor = admission.state?.actors[0]
    if (!actor || actor.type !== 'visible') throw new Error('expected visible actor')
    actor.inventory.equipment = 'wallPass'
    expect(validateGameplayV10ServerMessage(admission)).toEqual({ valid: true, value: admission })

    const roomState = { type: 'roomState' as const, room: admission.room }
    expect(validateGameplayV10ServerMessage(roomState)).toEqual({ valid: true, value: roomState })
  })

  it('accepts a nonholder overlapping a soft block without exposing private exit grace', () => {
    const message = withMarker(null)
    const actor = message.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible actor')
    actor.position = { x: 2560, y: 2560 }
    message.state.arena.blocks = [{ column: 2, row: 2 }]
    expect(validateGameplayV10ServerMessage(message)).toEqual({ valid: true, value: message })

    const leaked = structuredClone(message) as unknown as Record<string, unknown>
    const leakedActors = (leaked.state as { actors: Array<Record<string, unknown>> }).actors
    leakedActors[0].wallPassExitGrace = [{ column: 2, row: 2 }]
    expect(validateGameplayV10ServerMessage(leaked)).toMatchObject({ valid: false, source: 'schema' })
  })

  it.each(supportedRanges)('accepts every supported enabled-item mask in [$start, $end)', (range) => {
    executeCatalogMaskRange('gameplay v10', supported, range, (_mask, enabledItems) => {
      const message = withMarker(null)
      message.room.settings.enabledItems = [...enabledItems]
      expect(validateGameplayV10ServerMessage(message)).toEqual({ valid: true, value: message })
    })
  })

  it('accepts equal visible markers only inside the current physical round', () => {
    const message = withMarker(1)
    message.room.settings.enabledItems = ['teleporter']
    expect(validateGameplayV10ServerMessage(message)).toEqual({ valid: true, value: message })

    for (const marker of [0, 3]) {
      const invalid = withMarker(marker)
      invalid.room.settings.enabledItems = ['teleporter']
      expect(validateGameplayV10ServerMessage(invalid)).toEqual({
        valid: false, source: 'semantics', error: 'invalidTeleportMarker',
      })
    }
  })

  it('rejects disabled and unequal visible marker history', () => {
    const disabled = withMarker(1)
    expect(validateGameplayV10ServerMessage(disabled)).toEqual({
      valid: false, source: 'semantics', error: 'invalidTeleportMarker',
    })
    const unequal = withMarker(1)
    unequal.room.settings.enabledItems = ['teleporter']
    const rival = unequal.state.actors[1]
    if (rival.type !== 'visible') throw new Error('expected visible rival')
    rival.lastTeleportTick = null
    expect(validateGameplayV10ServerMessage(unequal)).toEqual({
      valid: false, source: 'semantics', error: 'invalidTeleportMarker',
    })
  })

  it('requires visible markers and keeps hidden actors at their exact four keys', () => {
    const missing = structuredClone(withMarker(null)) as unknown as Record<string, unknown>
    const actors = (missing.state as { actors: Array<Record<string, unknown>> }).actors
    delete actors[0].lastTeleportTick
    expect(validateGameplayV10ServerMessage(missing)).toMatchObject({ valid: false, source: 'schema' })

    const hidden = structuredClone(withMarker(null)) as unknown as Record<string, unknown>
    const hiddenActors = (hidden.state as { actors: Array<Record<string, unknown>> }).actors
    hiddenActors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true, lastTeleportTick: null }
    expect(validateGameplayV10ServerMessage(hidden)).toMatchObject({ valid: false, source: 'schema' })
  })
})
