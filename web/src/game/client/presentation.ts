import type { SnapshotMessage } from '../arena/types'
import type { GameClientState } from './clientTypes'

const renderedPickups = new Set([
  'bombCapacity', 'blastRange', 'movementSpeed', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'goldenFlame', 'shortFuse',
  'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls',
])

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
        .map(({ id, seat, position, heading, alive, damageGraceRemainingTicks, inventory, effects }) => ({
          id, seat, position, heading, alive, damageGraceRemainingTicks,
          remoteArmed: snapshot.state.bombs.some((bomb) =>
            bomb.owner === id && bomb.trigger.type === 'remote'),
          piercingArmed: inventory.weapon === 'piercingBomb',
          powerArmed: inventory.weapon === 'powerBomb',
          landmineStored: inventory.weapon === 'landmine',
          shieldRemainingTicks: effects.find((effect) => effect.kind === 'shield')?.remainingTicks ?? 0,
          goldenFlameRemainingTicks:
            effects.find((effect) => effect.kind === 'goldenFlame')?.remainingTicks ?? 0,
          shortFuseRemainingTicks:
            effects.find((effect) => effect.kind === 'shortFuse')?.remainingTicks ?? 0,
          diarrheaRemainingTicks:
            effects.find((effect) => effect.kind === 'diarrhea')?.remainingTicks ?? 0,
          slowFootRemainingTicks: effects.filter((effect) => effect.kind === 'slowFoot')
            .map((effect) => effect.remainingTicks),
          shortFlameRemainingTicks:
            effects.find((effect) => effect.kind === 'shortFlame')?.remainingTicks ?? 0,
          longFuseRemainingTicks:
            effects.find((effect) => effect.kind === 'longFuse')?.remainingTicks ?? 0,
          reverseControlsRemainingTicks:
            effects.find((effect) => effect.kind === 'reverseControls')?.remainingTicks ?? 0,
        })),
      bombs: bombsHidden ? [] : snapshot.state.bombs,
      flames: snapshot.state.flames,
      pickups: snapshot.state.pickups.filter(
        (pickup): pickup is typeof pickup & {
          kind: 'bombCapacity' | 'blastRange' | 'movementSpeed' | 'heart' | 'remoteBomb' | 'piercingBomb' | 'powerBomb' | 'landmine' | 'shield'
            | 'goldenFlame' | 'shortFuse' | 'diarrhea' | 'slowFoot' | 'shortFlame' | 'longFuse'
            | 'reverseControls'
        } =>
          renderedPickups.has(pickup.kind),
      ),
    },
  }
}
