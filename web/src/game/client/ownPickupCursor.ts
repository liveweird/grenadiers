import { RENDERABLE_PICKUP_KINDS, type RenderablePickupKind } from '../arena/types'
import type { GameClientState } from './clientTypes'

const RECENT_TICKS = 180
const renderedKinds = new Set<string>(RENDERABLE_PICKUP_KINDS)
export interface PickupFeedback { identity: string; kind: RenderablePickupKind }

export class OwnPickupCursor {
  private roomId: string | null = null
  private roundId: number | null = null
  private baseline: GameClientState['admissionBaseline'] = null
  private revision = -1
  private physicalTick = -1
  private recent: (PickupFeedback & { tick: number }) | null = null

  consume(state: GameClientState): PickupFeedback | null {
    const { room, snapshot } = state
    if (!room) { this.reset(); return null }
    if (!snapshot || !state.youSeat || state.connection !== 'authenticated' || state.admissionPending) {
      return this.clearRecent()
    }
    if (this.changedBoundary(state)) return this.seed(state)
    const own = snapshot.state.actors.find((actor) => actor.seat === state.youSeat)
    if (own?.type !== 'visible' || !own.alive) this.recent = null
    if (this.accepts(state)) this.collect(state)
    return own?.type === 'visible' && own.alive ? this.current(this.physicalTick) : null
  }

  private changedBoundary(state: GameClientState): boolean {
    const room = state.room!
    return room.roomId !== this.roomId || room.roundId !== this.roundId
      || state.admissionBaseline !== this.baseline
  }

  private seed(state: GameClientState): null {
    const room = state.room!
    const snapshot = state.snapshot!
    this.roomId = room.roomId
    this.roundId = room.roundId
    this.baseline = state.admissionBaseline
    this.revision = snapshot.room.revision
    this.physicalTick = snapshot.state.physicalTick
    return this.clearRecent()
  }

  private accepts(state: GameClientState): boolean {
    const room = state.room!
    const snapshot = state.snapshot!
    return snapshot.room.roomId === room.roomId && snapshot.room.roundId === room.roundId
      && snapshot.room.revision > this.revision && snapshot.state.physicalTick >= this.physicalTick
  }

  private collect(state: GameClientState): void {
    const snapshot = state.snapshot!
    const previousTick = this.physicalTick
    this.revision = snapshot.room.revision
    this.physicalTick = snapshot.state.physicalTick
    const identities = new Set<string>()
    for (const event of snapshot.events) {
      const identity = `${event.tick}:${event.index}`
      if (event.tick <= previousTick || event.tick > this.physicalTick || identities.has(identity)) continue
      identities.add(identity)
      if (event.kind.type === 'pickupCollected' && event.kind.seat === state.youSeat
        && event.kind.kind !== 'oilBarrel' && renderedKinds.has(event.kind.kind)) {
        this.recent = { identity, kind: event.kind.kind as RenderablePickupKind, tick: event.tick }
      }
    }
  }

  private current(tick: number): PickupFeedback | null {
    if (this.recent && tick - this.recent.tick <= RECENT_TICKS) {
      return { identity: this.recent.identity, kind: this.recent.kind }
    }
    return this.clearRecent()
  }

  private clearRecent(): null { this.recent = null; return null }
  private reset(): void {
    this.roomId = null
    this.roundId = null
    this.baseline = null
    this.revision = -1
    this.physicalTick = -1
    this.recent = null
  }
}
