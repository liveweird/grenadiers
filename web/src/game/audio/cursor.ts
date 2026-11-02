import type { GameClientState } from '../client/client'
import type { PresentationDeaths } from '../arena/types'
import type { EventKind } from '../v20/schema'

export type AudioCue = 'step' | 'placement' | 'specialPlacement' | 'detonation'
  | 'specialDetonation' | 'pickup' | 'specialPickup' | 'elimination'
const priority: readonly AudioCue[] = [
  'elimination', 'specialDetonation', 'detonation', 'specialPickup', 'pickup',
  'specialPlacement', 'placement', 'step',
]
const specialPickups = new Set(['remoteBomb', 'piercingBomb', 'powerBomb', 'landmine'])
const STEP_DISTANCE = 192
const STEP_TICKS = 6
const MAX_WALK_GAP_TICKS = 6
const MAX_WALK_UNITS_PER_TICK = 96

function cueForEvent(kind: EventKind): AudioCue | null {
  if (kind.type === 'actorEliminated') return 'elimination'
  if (kind.type === 'pickupCollected') return specialPickups.has(kind.kind) ? 'specialPickup' : 'pickup'
  if (kind.type !== 'bombPlaced' && kind.type !== 'bombDetonated') return null
  const special = kind.bomb.blast !== 'standard' || kind.bomb.trigger.type !== 'timed'
  if (kind.type === 'bombPlaced') return special ? 'specialPlacement' : 'placement'
  return special ? 'specialDetonation' : 'detonation'
}

function collectCues(state: GameClientState, previousTick: number): Set<AudioCue> {
  const found = new Set<AudioCue>()
  const identities = new Set<string>()
  const snapshot = state.snapshot!
  for (const event of snapshot.events) {
    const identity = `${event.tick}:${event.index}`
    if (event.tick <= previousTick || event.tick > snapshot.state.physicalTick || identities.has(identity)) continue
    identities.add(identity)
    const cue = cueForEvent(event.kind)
    if (cue) found.add(cue)
  }
  return found
}

type LocalPosition = { x: number; y: number; lastTeleportTick: number | null; alive: boolean }

function localPosition(state: GameClientState): LocalPosition | null {
  const actor = state.snapshot?.state.actors.find(({ seat }) => seat === state.youSeat)
  return actor?.type === 'visible' ? {
    x: actor.position.x, y: actor.position.y, lastTeleportTick: actor.lastTeleportTick, alive: actor.alive,
  } : null
}

export class AudioCursor {
  private roomId: string | null = null
  private roundId: number | null = null
  private baseline: GameClientState['admissionBaseline'] = null
  private revision = -1
  private completedTick = -1
  private audible = false
  private local: LocalPosition | null = null
  private lastStepTick = -Infinity
  private stepDistance = 0
  private deathEpoch: number | null = null
  private mobDeathKeys = new Set<string>()

  consume(state: GameClientState, audible: boolean, deaths?: PresentationDeaths): AudioCue[] {
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
      this.local = localPosition(state)
      this.lastStepTick = -Infinity
      this.stepDistance = 0
      this.seedDeaths(deaths)
      return []
    }
    if (!snapshot || snapshot.room.roomId !== room.roomId || snapshot.room.roundId !== room.roundId
      || snapshot.room.revision <= this.revision) return []
    const previousTick = this.completedTick
    this.revision = snapshot.room.revision
    this.completedTick = Math.max(this.completedTick, snapshot.state.physicalTick)
    const found = collectCues(state, previousTick)
    const nextLocal = localPosition(state)
    if (this.walked(nextLocal, snapshot.state.physicalTick, previousTick)) found.add('step')
    if (this.newMobDeath(deaths)) found.add('elimination')
    this.local = nextLocal
    return audible ? priority.filter((cue) => found.has(cue)) : []
  }

  private walked(next: LocalPosition | null, tick: number, previousTick: number): boolean {
    const prior = this.local
    if (!prior || !next || !prior.alive || !next.alive || next.lastTeleportTick !== prior.lastTeleportTick) {
      this.stepDistance = 0
      return false
    }
    const distance = Math.hypot(next.x - prior.x, next.y - prior.y)
    const elapsed = Math.max(1, tick - previousTick)
    if (elapsed > MAX_WALK_GAP_TICKS || distance > 512 || distance > elapsed * MAX_WALK_UNITS_PER_TICK) {
      this.stepDistance = 0
      return false
    }
    this.stepDistance += distance
    if (this.stepDistance < STEP_DISTANCE || tick - this.lastStepTick < STEP_TICKS) return false
    this.stepDistance = 0
    this.lastStepTick = tick
    return true
  }

  private changedBoundary(state: GameClientState, audible: boolean): boolean {
    const room = state.room!
    return room.roomId !== this.roomId || room.roundId !== this.roundId
      || state.admissionBaseline !== this.baseline || (audible && !this.audible)
  }

  private seedDeaths(deaths?: PresentationDeaths): void {
    this.deathEpoch = deaths?.epoch ?? null
    this.mobDeathKeys = new Set(deaths?.cues.filter(({ appearance }) => appearance === 'mob')
      .map(({ key }) => key) ?? [])
  }

  private newMobDeath(deaths?: PresentationDeaths): boolean {
    if (!deaths) return false
    if (this.deathEpoch !== deaths.epoch) {
      this.seedDeaths(deaths)
      return false
    }
    const keys = deaths.cues.filter(({ appearance }) => appearance === 'mob').map(({ key }) => key)
    const found = keys.some((key) => !this.mobDeathKeys.has(key))
    this.mobDeathKeys = new Set(keys)
    return found
  }

  private reset(): void {
    this.roomId = null
    this.roundId = null
    this.baseline = null
    this.revision = -1
    this.completedTick = -1
    this.audible = false
    this.local = null
    this.lastStepTick = -Infinity
    this.stepDistance = 0
    this.deathEpoch = null
    this.mobDeathKeys = new Set()
  }
}
