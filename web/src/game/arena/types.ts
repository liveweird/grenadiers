import type { KeyboardBindings } from './bindings'

export type RenderablePickupKind =
  | 'bombCapacity' | 'blastRange' | 'movementSpeed' | 'kick' | 'punch' | 'bracelet' | 'wallPass' | 'heart' | 'remoteBomb' | 'piercingBomb' | 'powerBomb' | 'landmine' | 'shield' | 'stopwatch' | 'invisibilityCloak' | 'goldenFlame' | 'shortFuse'
  | 'diarrhea' | 'slowFoot' | 'shortFlame' | 'longFuse' | 'reverseControls' | 'invisibleBombs' | 'teleporter'

function pickupKinds<const T extends readonly RenderablePickupKind[]>(
  values: T & ([RenderablePickupKind] extends [T[number]] ? unknown : never),
): T { return values }

export const RENDERABLE_PICKUP_KINDS = pickupKinds([
  'bombCapacity', 'blastRange', 'movementSpeed', 'kick', 'punch', 'bracelet', 'wallPass', 'heart',
  'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch',
  'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame',
  'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
] as const)
interface PresentedActor {
  id: number
  seat: Seat
  position: { x: number; y: number }
  heading: Direction
  alive: boolean
  damageGraceRemainingTicks?: number
  shieldRemainingTicks?: number
  stopwatchRemainingTicks?: number
  voluntaryActionsFrozen?: boolean
  invisibilityCloakRemainingTicks?: number
  goldenFlameRemainingTicks?: number
  shortFuseRemainingTicks?: number
  diarrheaRemainingTicks?: number
  slowFootRemainingTicks?: number[]
  shortFlameRemainingTicks?: number
  longFuseRemainingTicks?: number
  reverseControlsRemainingTicks?: number
  invisibleBombsRemainingTicks?: number
  lastTeleportTick?: number | null
  piercingArmed?: boolean
  powerArmed?: boolean
  landmineStored?: boolean
  remoteArmed?: boolean
  kickEquipped?: boolean
  punchEquipped?: boolean
  braceletEquipped?: boolean
  wallPassEquipped?: boolean
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
      width: 11 | 13 | 15
      height: 9 | 11 | 13
      terrain: Array<'floor' | 'wall'>
      blocks: Array<{ column: number; row: number }>
    }
    actors: PresentedActor[]
    bombs: PresentedBomb[]
    flames: Array<{ cell: { column: number; row: number }; expiresAt: number }>
    pickups: Array<{ cell: { column: number; row: number }; kind: RenderablePickupKind }>
    oilBarrels?: Array<{
      cell: { column: number; row: number }
      state: 'unlit' | 'burning'
    }>
  }
}
export type Seat = 'one' | 'two' | 'three' | 'four'
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
  onEquipment: () => void
}
