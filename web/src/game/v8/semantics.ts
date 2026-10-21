import type { BombView, EffectView, GameState, ItemId, RoomView, WireEvent } from './schema'

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

const supportedItems = [
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'shield', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse',
  'reverseControls',
] as const

function supportedItem(kind: string, enabled: ReadonlySet<ItemId>): boolean {
  return ['bombCapacity', 'blastRange', 'movementSpeed'].includes(kind)
    || (supportedItems.includes(kind as typeof supportedItems[number]) && enabled.has(kind as ItemId))
}

function supportedEffect(effect: EffectView, enabled: ReadonlySet<ItemId>): boolean {
  const limit = effect.kind === 'shield' ? 300 : 600
  return supportedItems.includes(effect.kind as typeof supportedItems[number])
    && enabled.has(effect.kind as ItemId)
    && effect.remainingTicks <= limit
}

function supportedBomb(bomb: BombView, enabled: ReadonlySet<ItemId>): boolean {
  if (bomb.trigger.type === 'remote') {
    return enabled.has('remoteBomb') && bomb.blast === 'standard'
  }
  return bomb.trigger.type === 'timed' && !bomb.trigger.paused
    && (bomb.blast === 'standard'
      || (bomb.blast === 'piercing' && enabled.has('piercingBomb'))
      || (bomb.blast === 'power' && enabled.has('powerBomb')))
}

function supportedState(state: GameState | null, enabled: ReadonlySet<ItemId>): boolean {
  if (!state) return true
  if (state.pickups.some((pickup) => !supportedItem(pickup.kind, enabled))) return false
  const actorIds = new Set(state.actors.map((actor) => actor.id))
  return state.bombs.every((bomb) => actorIds.has(bomb.owner) && supportedBomb(bomb, enabled))
    && state.bombs.filter((bomb) => bomb.trigger.type === 'remote')
      .every((bomb, index, remotes) => remotes.findIndex((other) => other.owner === bomb.owner) === index)
    && state.actors.every((actor) => actor.type === 'visible' && (
    (actor.inventory.weapon === null
      || (actor.inventory.weapon === 'piercingBomb' && enabled.has('piercingBomb'))
      || (actor.inventory.weapon === 'remoteBomb' && enabled.has('remoteBomb'))
      || (actor.inventory.weapon === 'powerBomb' && enabled.has('powerBomb')))
    && (actor.inventory.equipment === null
      || (actor.inventory.equipment === 'heart' && enabled.has('heart')))
    && actor.effects.every((effect) => supportedEffect(effect, enabled))
    && (enabled.has('heart') || actor.damageGraceRemainingTicks === 0)
  ))
}

function supportedEvents(events: readonly WireEvent[], enabled: ReadonlySet<ItemId>): boolean {
  return events.every(({ kind }) => {
    if (kind.type === 'pickupCollected') return supportedItem(kind.kind, enabled)
    if (kind.type === 'bombPlaced' || kind.type === 'bombDetonated') {
      return supportedBomb(kind.bomb, enabled)
    }
    return kind.type !== 'damagePrevented'
      || enabled.has(kind.source)
  })
}

export function expandedProfileSemanticError(
  room: RoomView, state: GameState | null, events: readonly WireEvent[] = [],
): GameplayV8SemanticError | null {
  const enabled = new Set(room.settings.enabledItems)
  const canonicalEnabled = supportedItems.filter((item) => enabled.has(item))
  if (room.rulesVersion !== 5 || room.mapId !== 'generated_v2'
    || room.availableItems.length !== supportedItems.length
    || room.availableItems.some((item, index) => item !== supportedItems[index])
    || room.settings.enabledItems.length !== canonicalEnabled.length
    || room.settings.enabledItems.some((item, index) => item !== canonicalEnabled[index])) {
    return 'unsupportedProfile'
  }
  return supportedState(state, enabled) && supportedEvents(events, enabled)
    ? null : 'unsupportedProfile'
}
