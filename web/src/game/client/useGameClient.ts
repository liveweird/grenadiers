import { useEffect, useState } from 'react'
import { GameClient, initialGameClientState, type GameClientState } from './client'
import type { PresentationDeaths } from '../arena/types'
import { DeathPresentationTracker } from './deathPresentation'

export function useGameClient(): readonly [GameClientState, GameClient, PresentationDeaths] {
  const [tracker] = useState(() => new DeathPresentationTracker())
  const [published, setPublished] = useState(() => {
    const state = initialGameClientState()
    return { state, deaths: tracker.consume(state, performance.now()) }
  })
  const [client] = useState(() => new GameClient((state) => {
    const deaths = tracker.consume(state, performance.now())
    setPublished({ state, deaths })
  }))
  useEffect(() => () => client.dispose(), [client])
  return [published.state, client, published.deaths]
}
