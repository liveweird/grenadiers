import { useSyncExternalStore } from 'react'
import { ArenaWorld } from './ArenaWorld'
import type { ArenaSceneStore, ThreeAtlas } from './contracts'
import { Entities } from './Entities'
import { FinishPass } from './finishPass'
import { readSceneTiming } from './sceneStore'
import { Deaths } from './Deaths'
import { BloodPools } from './BloodPools'
import { SpikePits } from './SpikePits'
import { FireIllumination } from './FireIllumination'

interface ArenaSceneProps {
  store: ArenaSceneStore
  atlas: ThreeAtlas
}

export function ArenaScene({ store, atlas }: ArenaSceneProps) {
  const revision = useSyncExternalStore(store.subscribe, store.read, store.read)
  const timing = readSceneTiming(store)

  if (!revision.snapshot || !timing || !revision.fire) return null
  return <>
    <ArenaWorld snapshot={revision.snapshot} atlas={atlas}>
      <FireIllumination model={revision.fire.lighting} animated={timing.running && !revision.reducedMotion}>
      <SpikePits pits={revision.snapshot.state.spikePits} atlas={atlas}
        reducedMotion={revision.reducedMotion} />
      <Entities
        snapshot={revision.snapshot}
        atlas={atlas}
        timing={timing}
        reducedMotion={revision.reducedMotion}
        fire={revision.fire}
      />
      <BloodPools pools={revision.bloodPools ?? []} atlas={atlas} />
      <Deaths cues={revision.deaths} atlas={atlas} reducedMotion={revision.reducedMotion} />
      </FireIllumination>
    </ArenaWorld>
    <FinishPass />
  </>
}
