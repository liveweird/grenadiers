import { selectBombAnimation } from '../bombAnimation'
import type { SnapshotMessage } from '../types'
import type { GroundPosition } from './motion'
import { unitsToGround } from './motion'

type Bomb = SnapshotMessage['state']['bombs'][number]
type Cell = { column: number; row: number }
export interface BombView {
  shellFrame: string
  shellScale: number
  spark: boolean
  paused: boolean
  groundPosition: GroundPosition
}

export const cellCenter = (cell: Cell): GroundPosition => ({
  x: cell.column + 0.5,
  z: cell.row + 0.5,
})
export const cellKey = (cell: Cell): string => `${cell.column},${cell.row}`
export const pickupFrame = (kind: SnapshotMessage['state']['pickups'][number]['kind']): string =>
  `pickup.${kind}`

export function bombView(bomb: Bomb, animated: boolean): BombView {
  const trigger = bomb.trigger?.type ?? 'timed'
  const blast = bomb.blast ?? 'standard'
  const timed = bomb.trigger?.type === 'timed' ? bomb.trigger : null
  const paused = Boolean(timed?.paused)
  const remaining = timed?.remainingTicks ?? null
  const pose = selectBombAnimation(remaining, animated && !paused && trigger === 'timed')
  const suffix = pose.pose === 0 ? '' : `.${pose.pose}`
  return {
    shellFrame: `bomb.shell.${trigger}.${blast}${trigger === 'timed' ? suffix : ''}`,
    shellScale: pose.shellScale,
    spark: trigger === 'timed' && !paused && pose.sparkVisible,
    paused,
    groundPosition: cellCenter(bomb.cell),
  }
}

export const mobPosition = (mob: SnapshotMessage['state']['mobs'][number]): GroundPosition =>
  unitsToGround(mob.position)
export const cloudPosition = (
  cloud: NonNullable<SnapshotMessage['state']['stormCloud']>,
): GroundPosition => unitsToGround(cloud.position)
