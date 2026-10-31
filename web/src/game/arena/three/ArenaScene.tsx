import { useSyncExternalStore } from 'react'
import { ArenaWorld } from './ArenaWorld'
import type { ArenaSceneStore, ThreeAtlas } from './contracts'
import { Entities } from './Entities'
import { FinishPass } from './finishPass'
import { readSceneTiming } from './sceneStore'
import { Deaths } from './Deaths'

interface ArenaSceneProps {
  store: ArenaSceneStore
  atlas: ThreeAtlas
}

export function ArenaScene({ store, atlas }: ArenaSceneProps) {
  const revision = useSyncExternalStore(store.subscribe, store.read, store.read)
  const timing = readSceneTiming(store)

  if (!revision.snapshot || !timing) return null
  return <>
    <ArenaWorld snapshot={revision.snapshot} atlas={atlas}>
      <Entities
        snapshot={revision.snapshot}
        atlas={atlas}
        timing={timing}
        reducedMotion={revision.reducedMotion}
      />
      <Deaths cues={revision.deaths} atlas={atlas} reducedMotion={revision.reducedMotion} />
    </ArenaWorld>
    <FinishPass />
  </>
}
