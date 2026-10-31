import { useEffect, useState } from 'react'
import { GameClient, initialGameClientState, type GameClientState } from './client'
import type { PresentationDeaths, PresentationOpening } from '../arena/types'
import { DeathPresentationTracker } from './deathPresentation'
import { RoundOpeningTracker } from './roundOpening'

export function useGameClient(): readonly [
  GameClientState, GameClient, PresentationDeaths, PresentationOpening | null,
] {
  const [tracker] = useState(() => new DeathPresentationTracker())
  const [openingTracker] = useState(() => new RoundOpeningTracker())
  const [published, setPublished] = useState(() => {
    const state = initialGameClientState()
    const now = performance.now()
    return {
      state, deaths: tracker.consume(state, now), opening: openingTracker.consume(state, now),
    }
  })
  const [client] = useState(() => new GameClient((state) => {
    const now = performance.now()
    const deaths = tracker.consume(state, now)
    const opening = openingTracker.consume(state, now)
    setPublished({ state, deaths, opening })
  }))
  useEffect(() => () => client.dispose(), [client])
  return [published.state, client, published.deaths, published.opening]
}
