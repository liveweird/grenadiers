import type { RoomView, Seat, V4ErrorCode } from './schema'
import type { Snapshot } from './schema'

type GameConnection =
  | 'disconnected'
  | 'connecting'
  | 'authenticating'
  | 'authenticated'
  | 'reconnecting'

type LocalFailure =
  | 'connectionFailed'
  | 'invalidServerMessage'
  | 'sessionFailed'
  | 'sequenceExhausted'

export type GameFailure = V4ErrorCode | LocalFailure | 'incompatibleProtocol' | 'seatReplaced'
export type AdmissionIntent = { type: 'create' } | { type: 'join'; roomCode: string }

export interface GameClientState {
  connection: GameConnection
  room: RoomView | null
  snapshot: Snapshot | null
  youSeat: Seat | null
  failure: GameFailure | null
  admissionPending: boolean
  leavePending: boolean
  reconnectAttempt: number
}

export const initialGameClientState = (): GameClientState => ({
  connection: 'disconnected', room: null, snapshot: null, youSeat: null, failure: null,
  admissionPending: false, leavePending: false, reconnectAttempt: 0,
})
