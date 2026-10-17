import type { GameState } from '../schema'

// Validated wire versions share this presentation boundary. The arena does not
// interpret scores, match outcomes, event envelopes, or transport versions.
export interface SnapshotMessage {
  room: {
    roomId: string
    roundId: number
    phase: { type: string }
  }
  state: GameState
}
export type Seat = 'one' | 'two'
export type Direction = 'north' | 'east' | 'south' | 'west'

export interface GameArenaProps {
  label: string
  waitingText: string
  failureText: string
  focusHint: string
  instructionsId: string
  snapshot: SnapshotMessage | null
  youSeat: Seat | null
  enabled: boolean
  onDirection: (direction: Direction | null) => void
  onBomb: () => void
}
