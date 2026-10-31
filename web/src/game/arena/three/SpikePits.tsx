import type { SnapshotMessage } from '../types'
import { Billboard } from './Billboard'
import { cellToWorld } from './coordinates'
import type { ThreeAtlas } from './contracts'

const PIT_Y = 0.003
const MAX_RENDERED_PITS = 4

export function SpikePits({ pits, atlas, reducedMotion }: {
  pits: SnapshotMessage['state']['spikePits']
  atlas: ThreeAtlas
  reducedMotion: boolean
}) {
  return <group name="spike-pits">
    {pits.slice(0, MAX_RENDERED_PITS).map((pit) => {
      const [x, z] = cellToWorld(pit.cell.column, pit.cell.row)
      const warningOpacity = !reducedMotion && pit.phase === 'warning'
        ? (now: number) => 0.78 + Math.sin(now / 90) * 0.12
        : undefined
      return <group key={`${pit.cell.column}:${pit.cell.row}`} name={`spike-pit:${pit.phase}`}
        position={[x, PIT_Y, z]}>
        <Billboard atlas={atlas} frameId={`terrain.spikePit.${pit.phase}`} anchor="center"
          rotation={[-Math.PI / 2, 0, 0]} faceCamera={false} depthTest renderOrder={0}
          scale={0.98 / 48} {...(warningOpacity ? { opacityAt: warningOpacity } : {})} />
      </group>
    })}
  </group>
}
