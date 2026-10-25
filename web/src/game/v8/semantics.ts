import { ITEM_IDS, type GameState, type ItemId, type RoomView, type WireEvent } from './schema'
import {
  effectSemanticError,
  expandedProfileSemanticErrorFor,
  itemSetSemanticError,
  type GameplayProfileSemanticError,
} from '../validation/profileSemantics'

export type GameplayV8SemanticError = GameplayProfileSemanticError
export { effectSemanticError, itemSetSemanticError }

const activeItemIds = new Set<ItemId>([
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls',
] satisfies readonly ItemId[])
const supportedItems = ITEM_IDS.filter((item) => activeItemIds.has(item))

export function expandedProfileSemanticError(
  room: RoomView, state: GameState | null, events: readonly WireEvent[] = [],
): GameplayV8SemanticError | null {
  return expandedProfileSemanticErrorFor(room, state, events, supportedItems, false)
}
