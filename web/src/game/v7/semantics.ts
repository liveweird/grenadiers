import type { EffectView, ItemId } from './schema'

export type EffectSemanticError = 'duplicateEffect' | 'slowFootLimit'
export type ItemSetSemanticError = 'unavailableItem'
export type GameplayV7SemanticError = EffectSemanticError | ItemSetSemanticError | 'unsupportedProfile'

export function effectSemanticError(effects: readonly EffectView[]): EffectSemanticError | null {
  const ordinary = new Set<string>()
  let slowFootCount = 0

  for (const effect of effects) {
    if (effect.kind === 'slowFoot') {
      slowFootCount += 1
      if (slowFootCount > 6) return 'slowFootLimit'
    } else if (ordinary.has(effect.kind)) {
      return 'duplicateEffect'
    } else {
      ordinary.add(effect.kind)
    }
  }
  return null
}

export function itemSetSemanticError(
  enabledItems: readonly ItemId[],
  availableItems: readonly ItemId[],
): ItemSetSemanticError | null {
  const available = new Set(availableItems)
  return enabledItems.every((item) => available.has(item)) ? null : 'unavailableItem'
}
