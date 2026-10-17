import { useEffect, useState } from 'react'
import { GameClient, type GameClientState } from './client'

const initialState: GameClientState = {
  connection: 'disconnected',
  room: null,
  snapshot: null,
  youSeat: null,
  failure: null,
  admissionPending: false,
  leavePending: false,
}

export function useGameClient(): readonly [GameClientState, GameClient] {
  const [state, setState] = useState(initialState)
  const [client] = useState(() => new GameClient(setState))

  useEffect(() => () => client.dispose(), [client])
  return [state, client]
}
