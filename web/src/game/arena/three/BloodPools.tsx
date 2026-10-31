import type { BloodPool } from './bloodPoolState'
import { Billboard } from './Billboard'
import type { ThreeAtlas } from './contracts'
import { unitsToGround } from './motion'

export function BloodPools({ pools, atlas }: {
  pools: readonly BloodPool[]
  atlas: ThreeAtlas
}) {
  return <group name="blood-pools">
    {pools.map((pool) => {
      const ground = unitsToGround(pool.position)
      return <group key={pool.key} name={pool.key} position={[ground.x, 0.005, ground.z]}>
        <Billboard
          atlas={atlas} frameId="effect.bloodPool" anchor="center"
          rotation={[-Math.PI / 2, 0, 0]} faceCamera={false}
          depthTest renderOrder={1} scale={0.65 / 48}
        />
      </group>
    })}
  </group>
}
