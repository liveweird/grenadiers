import type { PresentationDeathCue, SnapshotMessage } from '../types'

const MAX_SEEN_CUES = 128

export interface BloodPool {
  key: string
  column: number
  row: number
  position: { x: number; y: number }
}

export interface BloodPoolState {
  pools: readonly BloodPool[]
  identity: string | null
  epoch: number | null
  sequence: number
  tick: number
  seen: ReadonlySet<string>
}

export const initialBloodPoolState = (): BloodPoolState => ({
  pools: [], identity: null, epoch: null, sequence: -1, tick: -1, seen: new Set(),
})

const identityOf = (snapshot: SnapshotMessage): string => [
  snapshot.room.roomId,
  snapshot.room.roundId,
  snapshot.room.presentationBaseline ?? 'legacy',
].join(':')

const tickOf = (snapshot: SnapshotMessage): number =>
  snapshot.state.physicalTick ?? snapshot.state.runningUpdates ?? 0

const boundedKeys = (cues: readonly PresentationDeathCue[]): Set<string> =>
  new Set(cues.slice(-MAX_SEEN_CUES).map(({ key }) => key))

const cellFor = (
  cue: PresentationDeathCue, snapshot: SnapshotMessage,
): { column: number; row: number } | null => {
  if (!Number.isFinite(cue.position.x) || !Number.isFinite(cue.position.y)) return null
  const column = Math.floor(cue.position.x / 1024)
  const row = Math.floor(cue.position.y / 1024)
  const { width, height } = snapshot.state.arena
  return column >= 0 && row >= 0 && column < width && row < height ? { column, row } : null
}

const poolsFrom = (
  snapshot: SnapshotMessage, cues: readonly PresentationDeathCue[],
): readonly BloodPool[] => {
  const byCell = new Map<string, BloodPool>()
  for (const cue of cues) {
    const cell = cellFor(cue, snapshot)
    if (!cell) continue
    const cellKey = `${cell.column}:${cell.row}`
    byCell.delete(cellKey)
    byCell.set(cellKey, {
      key: `blood:${cellKey}`, ...cell, position: { ...cue.position },
    })
  }
  const maximum = snapshot.state.arena.width * snapshot.state.arena.height
  return [...byCell.values()].slice(-maximum)
}

const establish = (snapshot: SnapshotMessage, acceptCues: boolean): BloodPoolState => {
  const presentation = snapshot.presentationDeaths
  if (!presentation) return initialBloodPoolState()
  return {
    pools: acceptCues ? poolsFrom(snapshot, presentation.cues) : [],
    identity: identityOf(snapshot),
    epoch: presentation.epoch,
    sequence: presentation.sequence,
    tick: tickOf(snapshot),
    seen: boundedKeys(presentation.cues),
  }
}

const appendCues = (
  snapshot: SnapshotMessage, pools: readonly BloodPool[], priorSeen: ReadonlySet<string>,
  cues: readonly PresentationDeathCue[],
): { pools: readonly BloodPool[]; seen: ReadonlySet<string> } => {
  const seen = new Set(priorSeen)
  const byCell = new Map(pools.map((pool) => [`${pool.column}:${pool.row}`, pool]))
  for (const cue of cues) {
    if (seen.has(cue.key)) continue
    seen.add(cue.key)
    if (seen.size > MAX_SEEN_CUES) seen.delete(seen.values().next().value as string)
    const cell = cellFor(cue, snapshot)
    if (!cell) continue
    const cellKey = `${cell.column}:${cell.row}`
    byCell.delete(cellKey)
    byCell.set(cellKey, { key: `blood:${cellKey}`, ...cell, position: { ...cue.position } })
  }
  const maximum = snapshot.state.arena.width * snapshot.state.arena.height
  return { pools: [...byCell.values()].slice(-maximum), seen }
}

/** Drops decals but retains the cue fence, preventing retained envelopes replaying after suspension. */
export const clearBloodPools = (state: BloodPoolState): BloodPoolState => ({
  ...state, pools: [],
})

/** Accepts only fresh, forward-moving recipient-safe cues; never derives positions from entities. */
export function advanceBloodPools(
  state: BloodPoolState, snapshot: SnapshotMessage,
): BloodPoolState {
  const presentation = snapshot.presentationDeaths
  if (!presentation) return clearBloodPools(state)
  const identity = identityOf(snapshot)
  const tick = tickOf(snapshot)
  if (state.identity === null) return establish(snapshot, true)
  if (state.identity !== identity || state.epoch !== presentation.epoch) {
    return establish(snapshot, state.epoch === null || presentation.epoch > state.epoch)
  }
  if (presentation.sequence < state.sequence || tick < state.tick) {
    return establish(snapshot, false)
  }

  const appended = presentation.sequence > state.sequence
    ? appendCues(snapshot, state.pools, state.seen, presentation.cues)
    : { pools: state.pools, seen: state.seen }
  return {
    pools: appended.pools, identity, epoch: presentation.epoch,
    sequence: Math.max(state.sequence, presentation.sequence), tick, seen: appended.seen,
  }
}
