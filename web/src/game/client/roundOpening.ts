import type { PresentationOpening } from '../arena/types'
import type { GameClientState } from './clientTypes'

const OPENING_LIFETIME_MS = 900

interface Observation {
  admission: string
  roomId: string
  roundId: number
  revision: number
  tick: number
  phase: string
}

const admissionKey = (state: GameClientState): string | null => {
  const baseline = state.admissionBaseline
  if (!baseline) return null
  return `${baseline.roomId}:${baseline.revision}:${baseline.tick}:${baseline.physicalTick ?? 'none'}`
}
const eligibleAdmission = (state: GameClientState): string | null =>
  state.connection === 'authenticated' && !state.admissionPending && state.room
    ? admissionKey(state) : null

const observation = (state: GameClientState, admission: string): Observation => ({
  admission,
  roomId: state.room!.roomId,
  roundId: state.room!.roundId,
  revision: state.room!.revision,
  tick: state.room!.tick,
  phase: state.room!.phase.type,
})

const continuous = (old: Observation, next: Observation): boolean =>
  old.admission === next.admission && old.roomId === next.roomId
  && next.roundId >= old.roundId && next.revision >= old.revision && next.tick >= old.tick

/** Latches the real room countdown before the first running snapshot reaches React. */
export class RoundOpeningTracker {
  private previous: Observation | null = null
  private pendingRound: number | null = null
  private cue: PresentationOpening | null = null

  consume(state: GameClientState, now: number): PresentationOpening | null {
    const admission = eligibleAdmission(state)
    if (!admission) {
      this.clear()
      return null
    }
    const next = observation(state, admission)
    const old = this.previous
    if (!old) {
      this.previous = next
      return null
    }
    if (!continuous(old, next)) {
      this.clear()
      this.previous = next
      return null
    }
    this.previous = next
    if (this.cue && (next.phase !== 'running' || next.roundId !== old.roundId
      || now - this.cue.bornAt >= OPENING_LIFETIME_MS)) this.cue = null
    if (next.phase === 'countdown') {
      if (old.phase !== 'countdown' || next.roundId > old.roundId) {
        this.pendingRound = next.roundId
      }
      return null
    }
    if (next.phase === 'running' && this.pendingRound === next.roundId) {
      const snapshot = state.snapshot
      const hasRunningWorld = snapshot?.room.roomId === next.roomId
        && snapshot.room.roundId === next.roundId && snapshot.room.phase.type === 'running'
      if (!hasRunningWorld) return this.cue
      this.cue = { key: `${next.admission}:${next.roundId}`, bornAt: now }
      this.pendingRound = null
    } else if (next.phase !== 'running') {
      this.pendingRound = null
    }
    return this.cue
  }

  private clear(): void {
    this.previous = null
    this.pendingRound = null
    this.cue = null
  }
}
