export type GameplayProfileSemanticError =
  | 'duplicateEffect'
  | 'slowFootLimit'
  | 'unavailableItem'
  | 'unsupportedProfile'

type SemanticEffect = { kind: string; remainingTicks: number }
type SemanticBomb = {
  owner: number
  blast: string
  trigger:
    | { type: 'timed'; remainingTicks: number; paused: boolean }
    | { type: 'remote' }
    | { type: 'contact' }
}
type SemanticActor = {
  type: 'visible'
  id: number
  alive: boolean
  inventory: { equipment: string | null; weapon: string | null }
  effects: readonly SemanticEffect[]
  damageGraceRemainingTicks: number
} | {
  type: 'hidden'
  id: number
  alive: boolean
}
type SemanticState = {
  pickups: readonly { kind: string }[]
  actors: readonly SemanticActor[]
  bombs: readonly SemanticBomb[]
}
type SemanticRoom = {
  rulesVersion: number
  mapId: string
  availableItems: readonly string[]
  settings: { enabledItems: readonly string[] }
}
type SemanticEvent = {
  kind: {
    type: string
    kind?: string
    bomb?: SemanticBomb
    source?: string
  }
}

export function effectSemanticError(
  effects: readonly SemanticEffect[],
): GameplayProfileSemanticError | null {
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
  enabledItems: readonly string[], availableItems: readonly string[],
): GameplayProfileSemanticError | null {
  const available = new Set(availableItems)
  return enabledItems.every((item) => available.has(item)) ? null : 'unavailableItem'
}

function supportedItem(
  kind: string, enabled: ReadonlySet<string>, catalog: readonly string[],
): boolean {
  return ['bombCapacity', 'blastRange', 'movementSpeed'].includes(kind)
    || (catalog.includes(kind) && enabled.has(kind))
}

function supportedEffect(
  effect: SemanticEffect, enabled: ReadonlySet<string>, catalog: readonly string[],
): boolean {
  const limit = effect.kind === 'shield' ? 300 : effect.kind === 'stopwatch' ? 180 : 600
  return catalog.includes(effect.kind)
    && enabled.has(effect.kind)
    && effect.remainingTicks <= limit
}

function supportedBomb(
  bomb: SemanticBomb, enabled: ReadonlySet<string>, allowPausedTimed: boolean,
): boolean {
  if (bomb.trigger.type === 'remote') {
    return enabled.has('remoteBomb') && bomb.blast === 'standard'
  }
  if (bomb.trigger.type === 'contact') {
    return enabled.has('landmine') && bomb.blast === 'standard'
  }
  return (!bomb.trigger.paused || (allowPausedTimed && enabled.has('stopwatch')))
    && (bomb.blast === 'standard'
      || (bomb.blast === 'piercing' && enabled.has('piercingBomb'))
      || (bomb.blast === 'power' && enabled.has('powerBomb')))
}

function supportedState(
  state: SemanticState | null, enabled: ReadonlySet<string>, catalog: readonly string[],
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
      || (catalog.includes(actor.inventory.equipment) && enabled.has(actor.inventory.equipment)))
    && actor.effects.every((effect) => supportedEffect(effect, enabled, catalog))
    && (enabled.has('heart') || actor.damageGraceRemainingTicks === 0))
}

function supportedEvents(
  events: readonly SemanticEvent[], enabled: ReadonlySet<string>, catalog: readonly string[],
  allowPausedTimed: boolean,
): boolean {
  return events.every(({ kind }) => {
    if (kind.type === 'pickupCollected') {
      return typeof kind.kind === 'string' && supportedItem(kind.kind, enabled, catalog)
    }
    if (kind.type === 'bombPlaced' || kind.type === 'bombDetonated') {
      return kind.bomb !== undefined && supportedBomb(kind.bomb, enabled, allowPausedTimed)
    }
    return kind.type !== 'damagePrevented'
      || (typeof kind.source === 'string' && enabled.has(kind.source))
  })
}

export function expandedProfileSemanticErrorFor(
  room: SemanticRoom, state: SemanticState | null, events: readonly SemanticEvent[],
  catalog: readonly string[], allowPausedTimed: boolean,
): GameplayProfileSemanticError | null {
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
