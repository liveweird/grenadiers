import type { GameClientState } from '../client/client'

export type AudioCue = 'placement' | 'detonation' | 'pickup' | 'elimination'
const priority: readonly AudioCue[] = ['elimination', 'detonation', 'pickup', 'placement']
const cueByEvent = {
  bombPlaced: 'placement', bombDetonated: 'detonation',
  pickupCollected: 'pickup', actorEliminated: 'elimination',
} as const

function collectCues(state: GameClientState, previousTick: number): Set<AudioCue> {
  const found = new Set<AudioCue>()
  const identities = new Set<string>()
  const snapshot = state.snapshot!
  for (const event of snapshot.events) {
    const identity = `${event.tick}:${event.index}`
    if (event.tick <= previousTick || event.tick > snapshot.state.physicalTick || identities.has(identity)) continue
    identities.add(identity)
    const cue = cueByEvent[event.kind.type as keyof typeof cueByEvent]
    if (cue) found.add(cue)
  }
  return found
}

export class AudioCursor {
  private roomId: string | null = null
  private roundId: number | null = null
  private baseline: GameClientState['admissionBaseline'] = null
  private revision = -1
  private completedTick = -1
  private audible = false

  consume(state: GameClientState, audible: boolean): AudioCue[] {
    const room = state.room
    const snapshot = state.snapshot
    const boundary = !room || this.changedBoundary(state, audible)
    this.audible = audible
    if (!room) { this.reset(); return [] }
    if (boundary) {
      const changedRoom = room.roomId !== this.roomId
      this.roomId = room.roomId
      this.roundId = room.roundId
      this.baseline = state.admissionBaseline
      this.revision = Math.max(room.revision, snapshot?.room.revision ?? -1)
      const physicalTick = snapshot?.state.physicalTick ?? state.admissionBaseline?.physicalTick ?? -1
      this.completedTick = changedRoom ? physicalTick : Math.max(this.completedTick, physicalTick)
      return []
    }
    if (!snapshot || snapshot.room.roomId !== room.roomId || snapshot.room.roundId !== room.roundId
      || snapshot.room.revision <= this.revision) return []
    const previousTick = this.completedTick
    this.revision = snapshot.room.revision
    this.completedTick = Math.max(this.completedTick, snapshot.state.physicalTick)
    const found = collectCues(state, previousTick)
    return audible ? priority.filter((cue) => found.has(cue)) : []
  }

  private changedBoundary(state: GameClientState, audible: boolean): boolean {
    const room = state.room!
    return room.roomId !== this.roomId || room.roundId !== this.roundId
      || state.admissionBaseline !== this.baseline || (audible && !this.audible)
  }

  private reset(): void {
    this.roomId = null
    this.roundId = null
    this.baseline = null
    this.revision = -1
    this.completedTick = -1
    this.audible = false
  }
}
