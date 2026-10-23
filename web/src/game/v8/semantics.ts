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
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse',
  'reverseControls',
] as const

function supportedItem(
  kind: string, enabled: ReadonlySet<ItemId>, catalog: readonly ItemId[],
): boolean {
  return ['bombCapacity', 'blastRange', 'movementSpeed'].includes(kind)
    || (catalog.includes(kind as ItemId) && enabled.has(kind as ItemId))
}

function supportedEffect(
  effect: EffectView, enabled: ReadonlySet<ItemId>, catalog: readonly ItemId[],
): boolean {
  const limit = effect.kind === 'shield' ? 300 : effect.kind === 'stopwatch' ? 180 : 600
  return catalog.includes(effect.kind as ItemId)
    && enabled.has(effect.kind as ItemId)
    && effect.remainingTicks <= limit
}

function supportedBomb(
  bomb: BombView, enabled: ReadonlySet<ItemId>, allowPausedTimed: boolean,
): boolean {
  if (bomb.trigger.type === 'remote') {
    return enabled.has('remoteBomb') && bomb.blast === 'standard'
  }
  if (bomb.trigger.type === 'contact') {
    return enabled.has('landmine') && bomb.blast === 'standard'
  }
  return bomb.trigger.type === 'timed'
    && (!bomb.trigger.paused || (allowPausedTimed && enabled.has('stopwatch')))
    && (bomb.blast === 'standard'
      || (bomb.blast === 'piercing' && enabled.has('piercingBomb'))
      || (bomb.blast === 'power' && enabled.has('powerBomb')))
}

function supportedState(
  state: GameState | null, enabled: ReadonlySet<ItemId>, catalog: readonly ItemId[],
  allowPausedTimed: boolean,
): boolean {
  if (!state) return true
  if (state.pickups.some((pickup) => !supportedItem(pickup.kind, enabled, catalog))) return false
  const actorIds = new Set(state.actors.map((actor) => actor.id))
  return state.bombs.every((bomb) => actorIds.has(bomb.owner)
      && (bomb.trigger.type !== 'timed' || bomb.trigger.remainingTicks > 0)
      && supportedBomb(bomb, enabled, allowPausedTimed))
    && state.bombs.filter((bomb) => bomb.trigger.type === 'remote')
      .every((bomb, index, remotes) => remotes.findIndex((other) => other.owner === bomb.owner) === index)
    && state.actors.every((actor) => actor.type === 'hidden'
      ? enabled.has('invisibilityCloak') && actor.alive
      : (actor.inventory.weapon === null
      || (actor.inventory.weapon === 'piercingBomb' && enabled.has('piercingBomb'))
      || (actor.inventory.weapon === 'remoteBomb' && enabled.has('remoteBomb'))
      || (actor.inventory.weapon === 'powerBomb' && enabled.has('powerBomb'))
      || (actor.inventory.weapon === 'landmine' && enabled.has('landmine')))
    && (actor.inventory.equipment === null
      || (actor.inventory.equipment === 'wallPass' || actor.inventory.equipment === 'heart')
        && catalog.includes(actor.inventory.equipment)
        && enabled.has(actor.inventory.equipment))
    && actor.effects.every((effect) => supportedEffect(effect, enabled, catalog))
    && (enabled.has('heart') || actor.damageGraceRemainingTicks === 0))
}

function supportedEvents(
  events: readonly WireEvent[], enabled: ReadonlySet<ItemId>, catalog: readonly ItemId[],
  allowPausedTimed: boolean,
): boolean {
  return events.every(({ kind }) => {
    if (kind.type === 'pickupCollected') return supportedItem(kind.kind, enabled, catalog)
    if (kind.type === 'bombPlaced' || kind.type === 'bombDetonated') {
      return supportedBomb(kind.bomb, enabled, allowPausedTimed)
    }
    return kind.type !== 'damagePrevented'
      || enabled.has(kind.source)
  })
}

export function expandedProfileSemanticError(
  room: RoomView, state: GameState | null, events: readonly WireEvent[] = [],
): GameplayV8SemanticError | null {
  return expandedProfileSemanticErrorFor(room, state, events, supportedItems, false)
}

export function expandedProfileSemanticErrorFor(
  room: RoomView, state: GameState | null, events: readonly WireEvent[],
  catalog: readonly ItemId[], allowPausedTimed: boolean,
): GameplayV8SemanticError | null {
  const enabled = new Set(room.settings.enabledItems)
  const canonicalEnabled = catalog.filter((item) => enabled.has(item))
  if (room.rulesVersion !== 5 || room.mapId !== 'generated_v2'
    || room.availableItems.length !== catalog.length
    || room.availableItems.some((item, index) => item !== catalog[index])
    || room.settings.enabledItems.length !== canonicalEnabled.length
    || room.settings.enabledItems.some((item, index) => item !== canonicalEnabled[index])) {
    return 'unsupportedProfile'
  }
  return supportedState(state, enabled, catalog, allowPausedTimed)
    && supportedEvents(events, enabled, catalog, allowPausedTimed)
    ? null : 'unsupportedProfile'
}
