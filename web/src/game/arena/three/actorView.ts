import { WALK_CYCLE, WALK_FRAME_DURATION_MS } from '../actorAnimation'
import type { Direction, Seat, SnapshotMessage } from '../types'
import type { GroundPosition } from './motion'
import { unitsToGround } from './motion'

type ActorRow = SnapshotMessage['state']['actors'][number]
export interface ActorCue {
  frameId: string
  pulse?: { floor: number; step: number; periodMs: number }
  layout?: {
    /** Offset from the actor's feet in normalized 48 px actor-card units. */
    offset: readonly [x: number, y: number]
    /** Scale relative to a full-size actor cue. */
    scale: number
  }
}
export interface ActorView {
  identity: string
  id: number
  seat: Seat
  heading: Direction
  groundPosition: GroundPosition
  alive: boolean
  moving: boolean
  frozen: boolean
  teleportMarker: number | null | undefined
  cues: readonly ActorCue[]
}

const active = (remaining: number | undefined): boolean => (remaining ?? 0) > 0
const cue = (
  frameId: string, enabled: boolean,
  pulse?: ActorCue['pulse'],
  layout?: ActorCue['layout'],
): ActorCue | null => enabled
  ? { frameId, ...(pulse ? { pulse } : {}), ...(layout ? { layout } : {}) }
  : null

const REMOTE_BADGE: NonNullable<ActorCue['layout']> = {
  offset: [22 / 48, 51 / 48],
  scale: 0.3,
}
const INVISIBLE_BADGE: NonNullable<ActorCue['layout']> = {
  offset: [-22 / 48, 51 / 48],
  scale: 0.3,
}

export function actorCues(actor: ActorRow): ActorCue[] {
  const slowStacks = actor.slowFootRemainingTicks?.length ?? 0
  const bombsInvisible = active(actor.invisibleBombsRemainingTicks)
  return [
    cue('actor.cue.protection', active(actor.damageGraceRemainingTicks), { floor: 0.65, step: 0.35, periodMs: 120 }),
    cue('actor.cue.shield', active(actor.shieldRemainingTicks), { floor: 0.75, step: 0.25, periodMs: 160 }),
    cue('actor.cue.stopwatch', active(actor.stopwatchRemainingTicks), { floor: 0.7, step: 0.3, periodMs: 120 }),
    cue('actor.cue.frozen', Boolean(actor.voluntaryActionsFrozen)),
    cue('actor.cue.goldenFlame', active(actor.goldenFlameRemainingTicks), { floor: 0.7, step: 0.3, periodMs: 100 }),
    cue('actor.cue.shortFuse', active(actor.shortFuseRemainingTicks), { floor: 0.65, step: 0.35, periodMs: 75 }),
    cue('actor.cue.shortFlame', active(actor.shortFlameRemainingTicks), { floor: 0.7, step: 0.3, periodMs: 130 }),
    cue('actor.cue.longFuse', active(actor.longFuseRemainingTicks), { floor: 0.65, step: 0.35, periodMs: 150 }),
    cue('actor.cue.reverseControls', active(actor.reverseControlsRemainingTicks), { floor: 0.7, step: 0.3, periodMs: 140 }),
    cue('actor.cue.diarrhea', active(actor.diarrheaRemainingTicks), { floor: 0.65, step: 0.35, periodMs: 105 }),
    cue('actor.cue.slowBoot', slowStacks > 0 || active(actor.stormSlowRemainingTicks)),
    ...Array.from({ length: Math.min(6, slowStacks) }, (_, index) => ({
      frameId: `actor.cue.slowMarker.${index + 1}`,
    })),
    cue('actor.cue.piercing', Boolean(actor.piercingArmed)),
    cue('actor.cue.power', Boolean(actor.powerArmed)),
    cue('actor.cue.landmine', Boolean(actor.landmineStored)),
    cue('actor.cue.kick', Boolean(actor.kickEquipped)),
    cue('actor.cue.punch', Boolean(actor.punchEquipped)),
    cue('actor.cue.bracelet', Boolean(actor.braceletEquipped)),
    cue('actor.cue.wallPass', Boolean(actor.wallPassEquipped)),
    cue('actor.cue.cloak', active(actor.invisibilityCloakRemainingTicks), { floor: 0.7, step: 0.3, periodMs: 140 }),
    cue('pickup.invisibleBombs', bombsInvisible, undefined, INVISIBLE_BADGE),
    cue('bomb.shell.remote.standard', Boolean(actor.remoteArmed), undefined, REMOTE_BADGE),
  ].filter((item): item is ActorCue => item !== null)
}

export function createActorView(actor: ActorRow, moving: boolean): ActorView {
  return {
    identity: `actor:${actor.id}`,
    id: actor.id,
    seat: actor.seat,
    heading: actor.heading,
    groundPosition: unitsToGround(actor.position),
    alive: actor.alive,
    moving,
    frozen: Boolean(actor.voluntaryActionsFrozen),
    teleportMarker: actor.lastTeleportTick,
    cues: actorCues(actor),
  }
}

export function actorFrameId(view: ActorView, now: number, animated: boolean): string {
  const frame = animated && view.moving && !view.frozen
    ? WALK_CYCLE[(Math.floor(now / WALK_FRAME_DURATION_MS) + view.id) % WALK_CYCLE.length]
    : 0
  return `actor.${view.seat}.${view.heading}.${frame}`
}

export function cueOpacity(cueView: ActorCue, now: number, animated: boolean): number {
  if (!animated || !cueView.pulse) return 1
  return cueView.pulse.floor + (Math.floor(now / cueView.pulse.periodMs) % 2) * cueView.pulse.step
}
