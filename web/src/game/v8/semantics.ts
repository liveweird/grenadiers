import type { EffectView, GameState, ItemId, RoomView, WireEvent } from './schema'

export type GameplayV8SemanticError =
  | 'duplicateEffect'
  | 'slowFootLimit'
  | 'unavailableItem'
  | 'unsupportedProfile'

export function effectSemanticError(effects: readonly EffectView[]): GameplayV8SemanticError | null {
  const ordinary = new Set<string>()
  let slowFootCount = 0
  for (const effect of effects) {
    if (effect.kind === 'slowFoot') {
      slowFootCount += 1
      if (slowFootCount > 6) return 'slowFootLimit'
    } else if (ordinary.has(effect.kind)) return 'duplicateEffect'
    else ordinary.add(effect.kind)
  }
  return null
}

export function itemSetSemanticError(
  enabledItems: readonly ItemId[], availableItems: readonly ItemId[],
): GameplayV8SemanticError | null {
  const available = new Set(availableItems)
  return enabledItems.every((item) => available.has(item)) ? null : 'unavailableItem'
}

function supportedItem(kind: string, heartEnabled: boolean): boolean {
  return ['bombCapacity', 'blastRange', 'movementSpeed'].includes(kind)
    || (kind === 'heart' && heartEnabled)
}

function supportedState(state: GameState | null, heartEnabled: boolean): boolean {
  if (!state) return true
  if (state.pickups.some((pickup) => !supportedItem(pickup.kind, heartEnabled))) return false
  return state.actors.every((actor) => actor.type === 'hidden' || (
    actor.inventory.weapon === null
    && (actor.inventory.equipment === null
      || (actor.inventory.equipment === 'heart' && heartEnabled))
    && actor.effects.length === 0
    && (heartEnabled || actor.damageGraceRemainingTicks === 0)
  ))
}

function supportedEvents(events: readonly WireEvent[], heartEnabled: boolean): boolean {
  return events.every(({ kind }) => {
    if (kind.type === 'pickupCollected') return supportedItem(kind.kind, heartEnabled)
    return kind.type !== 'damagePrevented' || (kind.source === 'heart' && heartEnabled)
  })
}

export function expandedProfileSemanticError(
  room: RoomView, state: GameState | null, events: readonly WireEvent[] = [],
): GameplayV8SemanticError | null {
  if (room.rulesVersion !== 5 || room.mapId !== 'generated_v2'
    || room.availableItems.length !== 1 || room.availableItems[0] !== 'heart'
    || room.settings.enabledItems.some((item) => item !== 'heart')) return 'unsupportedProfile'
  const heartEnabled = room.settings.enabledItems.includes('heart')
  return supportedState(state, heartEnabled) && supportedEvents(events, heartEnabled)
    ? null : 'unsupportedProfile'
}
