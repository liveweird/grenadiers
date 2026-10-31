import type { GameServerMessage } from '../../web/src/game/v18/schema'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
export type KickBombIdentity = Snapshot['state']['bombs'][number]

export type KickSnapshot = {
  room: { revision: number }
  state: { bombs: readonly KickBombIdentity[] }
}

const cellKey = ({ column, row }: KickBombIdentity['cell']): string => `${column},${row}`

/**
 * Retain the actually observed pre-input baseline and append only newer frames.
 * The exact authoritative route and immutable bomb identity remain fail-closed.
 */
export function retainedKickTimeline(
  baseline: KickSnapshot,
  observed: readonly KickSnapshot[],
  expected: KickBombIdentity,
): KickBombIdentity[] {
  const baselineBomb = baseline.state.bombs.find(({ id }) => id === expected.id)
  if (baselineBomb === undefined) throw new Error('kick baseline is missing the retained bomb')

  const timeline = [baselineBomb, ...observed
    .filter(({ room }) => room.revision > baseline.room.revision)
    .flatMap(({ state }) => state.bombs.filter(({ id }) => id === expected.id))]

  for (const bomb of timeline) {
    if (bomb.id !== expected.id || bomb.owner !== expected.owner || bomb.range !== expected.range
      || bomb.placedAt !== expected.placedAt || bomb.blast !== expected.blast
      || bomb.trigger.type !== 'timed' || bomb.trigger.paused !== false) {
      throw new Error('kick timeline changed retained bomb identity')
    }
  }
  const cells = timeline.map(({ cell }) => cellKey(cell))
    .filter((cell, index, all) => index === 0 || cell !== all[index - 1])
  if (cells.length !== 3 || cells[0] !== '3,1' || cells[1] !== '2,1' || cells[2] !== '1,1') {
    throw new Error(`kick timeline route mismatch: ${cells.join(' -> ')}`)
  }
  return timeline
}
