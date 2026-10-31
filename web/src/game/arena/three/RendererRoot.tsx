import { useLayoutEffect } from 'react'
import { ArenaScene } from './ArenaScene'
import type { ArenaSceneStore, ThreeAtlas } from './contracts'

export function RendererRoot({ atlas, store, resolve }: {
  atlas: ThreeAtlas
  store: ArenaSceneStore
  resolve: () => void
}) {
  useLayoutEffect(resolve, [resolve])
  return <ArenaScene atlas={atlas} store={store} />
}
