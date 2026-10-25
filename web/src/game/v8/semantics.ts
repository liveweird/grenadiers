import type { GameState, RoomView, WireEvent } from './schema'
import {
  effectSemanticError,
  expandedProfileSemanticErrorFor,
  itemSetSemanticError,
  type GameplayProfileSemanticError,
} from '../validation/profileSemantics'

export type GameplayV8SemanticError = GameplayProfileSemanticError
export { effectSemanticError, itemSetSemanticError }

const supportedItems = [
  'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls',
] as const

export function expandedProfileSemanticError(
  room: RoomView, state: GameState | null, events: readonly WireEvent[] = [],
): GameplayV8SemanticError | null {
  return expandedProfileSemanticErrorFor(room, state, events, supportedItems, false)
}
