import type { GameClientState } from './clientTypes'
import type { PresentationDeathCue, PresentationDeaths } from '../arena/types'
import type { Snapshot, VisibleActorView } from '../v17/schema'

const MAX_RETAINED_CUES = 64

interface SnapshotCursor {
  roomId: string
  roundId: number
  revision: number
  physicalTick: number
  receivedAt: number
  baseline: string
  phase: string
  mobs: Snapshot['state']['mobs']
  lairs: Snapshot['state']['lairs']
}

const admissionKey = (state: GameClientState): string | null => {
  const baseline = state.admissionBaseline
  if (!baseline) return null
  return `${baseline.roomId}:${baseline.revision}:${baseline.tick}:${baseline.physicalTick ?? 'none'}`
}

const visibleDeadActor = (
  snapshot: Snapshot, id: number, seat: VisibleActorView['seat'],
): VisibleActorView | null => {
  const actor = snapshot.state.actors.find((candidate) => candidate.id === id && candidate.seat === seat)
  return actor?.type === 'visible' && !actor.alive ? actor : null
}

const copyCue = ({ key, appearance, entityId, pose, tick, index, bornAt }: {
  key: string
  appearance: PresentationDeathCue['appearance']
  entityId: number
  pose: { position: { x: number; y: number }; heading: PresentationDeathCue['heading'] }
  tick: number
  index: number
  bornAt: number
}): PresentationDeathCue => ({
  key, appearance, entityId, position: { ...pose.position }, heading: pose.heading, tick, index, bornAt,
})

const cursorFor = (state: GameClientState, baseline: string, receivedAt: number): SnapshotCursor => {
  const snapshot = state.snapshot!
  return {
    roomId: snapshot.room.roomId,
    roundId: snapshot.room.roundId,
    revision: snapshot.room.revision,
    physicalTick: snapshot.state.physicalTick,
    receivedAt,
    baseline,
    phase: snapshot.room.phase.type,
    mobs: snapshot.state.mobs.map((mob) => ({ ...mob, position: { ...mob.position } })),
    lairs: snapshot.state.lairs.map((lair) => ({ ...lair, cell: { ...lair.cell } })),
  }
}

const continuous = (previous: SnapshotCursor, next: SnapshotCursor): boolean =>
  previous.roomId === next.roomId
  && previous.roundId === next.roundId
  && previous.baseline === next.baseline
  && next.revision >= previous.revision
  && next.physicalTick >= previous.physicalTick

/** Consumes every accepted client state before React scheduling can coalesce callbacks. */
export class DeathPresentationTracker {
  private previous: SnapshotCursor | null = null
  private active = false
  private epoch = 0
  private sequence = 0
  private cues: PresentationDeathCue[] = []
  private eliminatedActors = new Set<number>()

  consume(state: GameClientState, bornAt: number): PresentationDeaths {
    const baseline = admissionKey(state)
    const eligible = state.connection === 'authenticated' && !state.admissionPending
      && state.room !== null && state.snapshot !== null && baseline !== null
    if (!eligible) {
      if (this.active) this.reset()
      return this.read()
    }

    const next = cursorFor(state, baseline, bornAt)
    if (!this.previous) {
      this.establish(state.snapshot!, next)
      return this.read()
    }
    if (!continuous(this.previous, next)) {
      this.reset()
      this.establish(state.snapshot!, next)
      return this.read()
    }
    if (next.revision === this.previous.revision) return this.read()

    const emitted = [
      ...this.actorDeaths(state.snapshot!, this.previous, bornAt),
      ...this.mobDeaths(state.snapshot!, this.previous, next, bornAt),
    ]
    this.previous = next
    this.active = true
    if (emitted.length > 0) {
      this.cues = [...this.cues, ...emitted].slice(-MAX_RETAINED_CUES)
      this.sequence += 1
    }
    return this.read()
  }

  private actorDeaths(snapshot: Snapshot, previous: SnapshotCursor, bornAt: number): PresentationDeathCue[] {
    if (previous.phase !== 'running') return []
    return snapshot.events
      .filter((event) => event.tick >= previous.physicalTick
        && event.tick <= snapshot.state.physicalTick
        && event.kind.type === 'actorEliminated'
        && !this.eliminatedActors.has(event.kind.id))
      .flatMap((event) => {
        if (event.kind.type !== 'actorEliminated') return []
        this.eliminatedActors.add(event.kind.id)
        const actor = visibleDeadActor(snapshot, event.kind.id, event.kind.seat)
        if (!actor) return []
        return [copyCue({
          key: `actor:${event.tick}:${event.index}:${event.kind.id}`,
          appearance: event.kind.seat, entityId: event.kind.id, pose: actor,
          tick: event.tick, index: event.index, bornAt,
        })]
      })
  }

  private mobDeaths(
    snapshot: Snapshot, previous: SnapshotCursor, next: SnapshotCursor, bornAt: number,
  ): PresentationDeathCue[] {
    if (previous.phase !== 'running' || !['running', 'result', 'matchResult'].includes(next.phase)
      || next.physicalTick <= previous.physicalTick
      || next.physicalTick - previous.physicalTick > 60
      || next.receivedAt - previous.receivedAt > 1000) return []
    const currentIds = new Set(snapshot.state.mobs.map(({ id }) => id))
    const currentLairs = new Map(snapshot.state.lairs.map((lair) => [lair.id, lair.cell]))
    const previousLairs = new Map(previous.lairs.map((lair) => [lair.id, lair.cell]))
    return previous.mobs.flatMap((mob) => {
      const lair = currentLairs.get(mob.lairId)
      const oldLair = previousLairs.get(mob.lairId)
      if (currentIds.has(mob.id) || !lair || !oldLair
        || lair.column !== oldLair.column || lair.row !== oldLair.row) return []
      return [copyCue({
        key: `mob:${next.physicalTick}:${mob.id}`, appearance: 'mob', entityId: mob.id, pose: mob,
        tick: next.physicalTick, index: mob.id, bornAt,
      })]
    })
  }

  private reset(): void {
    this.previous = null
    this.active = false
    this.cues = []
    this.eliminatedActors = new Set()
    this.epoch += 1
    this.sequence += 1
  }

  private establish(snapshot: Snapshot, cursor: SnapshotCursor): void {
    this.previous = cursor
    this.active = true
    for (const event of snapshot.events) {
      if (event.kind.type === 'actorEliminated') this.eliminatedActors.add(event.kind.id)
    }
  }

  private read(): PresentationDeaths {
    return { epoch: this.epoch, sequence: this.sequence, cues: this.cues }
  }
}
