import type { SnapshotMessage } from '../arena/types'
import type { GameClientState } from './clientTypes'

const renderedPickups = new Set(['bombCapacity', 'blastRange', 'movementSpeed', 'heart'])

export function presentSnapshot(state: GameClientState): SnapshotMessage | null {
  const snapshot = state.snapshot
  if (!snapshot) return null
  const own = snapshot.state.actors.find((actor) => actor.seat === state.youSeat)
  const bombsHidden = own?.type === 'visible'
    && own.effects.some((effect) => effect.kind === 'invisibleBombs')
  const baseline = state.admissionBaseline
  return {
    room: {
      roomId: snapshot.room.roomId,
      roundId: snapshot.room.roundId,
      phase: snapshot.room.phase,
      presentationBaseline: baseline
        ? `${baseline.roomId}:${baseline.revision}:${baseline.tick}:${baseline.physicalTick ?? 'none'}`
        : 'none',
    },
    state: {
      physicalTick: snapshot.state.physicalTick,
      arena: snapshot.state.arena,
      actors: snapshot.state.actors
        .filter((actor) => actor.type === 'visible')
        .map(({ id, seat, position, heading, alive, damageGraceRemainingTicks }) => ({
          id, seat, position, heading, alive, damageGraceRemainingTicks,
        })),
      bombs: bombsHidden ? [] : snapshot.state.bombs,
      flames: snapshot.state.flames,
      pickups: snapshot.state.pickups.filter(
        (pickup): pickup is typeof pickup & {
          kind: 'bombCapacity' | 'blastRange' | 'movementSpeed' | 'heart'
        } =>
          renderedPickups.has(pickup.kind),
      ),
    },
  }
}
