import type { KeyboardBindings } from './bindings'

export type RenderablePickupKind =
  | 'bombCapacity' | 'blastRange' | 'movementSpeed' | 'heart' | 'shield' | 'goldenFlame' | 'shortFuse'
  | 'shortFlame' | 'longFuse' | 'reverseControls'
interface PresentedActor {
  id: number
  seat: Seat
  position: { x: number; y: number }
  heading: Direction
  alive: boolean
  damageGraceRemainingTicks?: number
  shieldRemainingTicks?: number
  goldenFlameRemainingTicks?: number
  shortFuseRemainingTicks?: number
  shortFlameRemainingTicks?: number
  longFuseRemainingTicks?: number
  reverseControlsRemainingTicks?: number
}
interface PresentedBomb {
  id: number
  cell: { column: number; row: number }
  owner: number
  range: number
  placedAt: number
  trigger?:
    | { type: 'timed'; remainingTicks: number; paused: boolean }
    | { type: 'remote' }
    | { type: 'contact' }
  blast?: 'standard' | 'piercing' | 'power'
}

export interface SnapshotMessage {
  room: {
    roomId: string
    roundId: number
    phase: { type: string }
    presentationBaseline?: string
  }
  state: {
    physicalTick?: number
    runningUpdates?: number
    arena: {
      width: 15
      height: 13
      terrain: Array<'floor' | 'wall'>
      blocks: Array<{ column: number; row: number }>
    }
    actors: PresentedActor[]
    bombs: PresentedBomb[]
    flames: Array<{ cell: { column: number; row: number }; expiresAt: number }>
    pickups: Array<{ cell: { column: number; row: number }; kind: RenderablePickupKind }>
  }
}
export type Seat = 'one' | 'two'
export type Direction = 'north' | 'east' | 'south' | 'west'

export interface GameArenaProps {
  label: string
  waitingText: string
  failureText: string
  focusHint: string
  instructionsId: string
  snapshot: SnapshotMessage | null
  youSeat: Seat | null
  enabled: boolean
  bindings?: KeyboardBindings
  onDirection: (direction: Direction | null) => void
  onBomb: () => void
}
