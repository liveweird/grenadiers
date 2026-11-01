import type { Difficulty, RoomConfiguration, RoomView, Seat, Snapshot, V14ErrorCode } from '../v19/schema'

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

export type GameFailure = V14ErrorCode | LocalFailure | 'incompatibleProtocol' | 'seatReplaced'
type CreatePreset = { type: 'humanDuel' } | { type: 'solo'; difficulty: Difficulty }
export type AdmissionIntent = { type: 'create'; mode?: CreatePreset; configuration?: RoomConfiguration }
  | { type: 'join'; roomCode: string }
type AdmissionBaseline = { roomId: string; revision: number; tick: number; physicalTick: number | null }

export interface GameClientState {
  connection: GameConnection
  room: RoomView | null
  snapshot: Snapshot | null
  youSeat: Seat | null
  failure: GameFailure | null
  admissionPending: boolean
  leavePending: boolean
  settingsPending: boolean
  reconnectAttempt: number
  admissionBaseline: AdmissionBaseline | null
}

export const initialGameClientState = (): GameClientState => ({
  connection: 'disconnected', room: null, snapshot: null, youSeat: null, failure: null,
  admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
  admissionBaseline: null,
})
