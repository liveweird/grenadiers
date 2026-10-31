import type { SnapshotMessage } from '../types'
import { cellKey } from './entityView'
import { createFireLighting, type FireLightingModel } from './fireLighting'
import type { MotionTiming } from './motion'

export const FIRE_IGNITION_MS = 160

export interface FireVisual {
  key: string
  column: number
  row: number
  bornAt: number | null
  oil: boolean
}
export interface FirePresentation {
  cells: readonly FireVisual[]
  lighting: FireLightingModel
  receivedAt: number
  tick: number
  running: boolean
}

/** Local age only animates public fire; it never starts or ends an authoritative flame. */
export function advanceFirePresentation(
  previous: FirePresentation | null | undefined,
  snapshot: SnapshotMessage,
  timing: MotionTiming,
  now: number,
): FirePresentation {
  const old = new Map(previous?.cells.map((cell) => [cell.key, cell]))
  const cells = new Map<string, FireVisual>()
  const tick = snapshot.state.physicalTick ?? 0
  const continuous = !timing.reset && previous != null
    && now >= previous.receivedAt && now - previous.receivedAt <= 250
    && tick >= previous.tick && tick - previous.tick <= 15
  const add = (column: number, row: number, stamp: string, oil: boolean): void => {
    const cell = { column, row }
    const key = `${cellKey(cell)}:${stamp}`
    const before = old.get(key)
    cells.set(cellKey(cell), {
      key, column, row, oil,
      bornAt: continuous ? before ? before.bornAt !== null && now - before.bornAt < FIRE_IGNITION_MS
        ? before.bornAt : null
        : timing.running || previous.running ? now : null : null,
    })
  }
  for (const flame of snapshot.state.flames.slice(0, 195)) {
    if (flame.expiresAt > tick) add(flame.cell.column, flame.cell.row, `${flame.expiresAt}`, false)
  }
  for (const barrel of snapshot.state.oilBarrels?.slice(0, 8) ?? []) {
    if (barrel.state === 'burning') add(barrel.cell.column, barrel.cell.row, 'oil', true)
  }
  return {
    cells: [...cells.values()].slice(0, 195), lighting: createFireLighting(snapshot),
    receivedAt: now, tick, running: timing.running,
  }
}
