import type { GameServerMessage } from '../schema'

export type SnapshotMessage = Extract<GameServerMessage, { type: 'snapshot' }>
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
