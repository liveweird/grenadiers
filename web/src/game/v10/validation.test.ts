import { describe, expect, it } from 'vitest'
import { checkedCatalogMaskRanges, executeCatalogMaskRange } from '../../test/catalogMasks'
import { snapshot } from '../client/testSupport'
import { validateGameplayV10ServerMessage } from './validation'

const supported = [
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
] as const
const supportedRanges = checkedCatalogMaskRanges({
  protocol: 'gameplay v10', catalog: supported, expectedCount: 17, expectedOrder: supported,
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

describe('gameplay v10 Teleporter validation', () => {
  it('pins seventeen unique settings entries before exhaustively enumerating all masks', () => {
    expect(supported).toHaveLength(17)
    expect(new Set(supported)).toHaveLength(17)
    expect(2 ** supported.length).toBe(131_072)
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
