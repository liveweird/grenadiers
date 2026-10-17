import { useEffect, useState } from 'react'
import { GameClient, initialGameClientState, type GameClientState } from './client'

export function useGameClient(): readonly [GameClientState, GameClient] {
  const [state, setState] = useState(initialGameClientState)
  const [client] = useState(() => new GameClient(setState))

  useEffect(() => () => client.dispose(), [client])
  return [state, client]
}
