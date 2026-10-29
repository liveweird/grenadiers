import type { ActorPoseFrame } from './actorArt'

export const WALK_FRAME_DURATION_MS = 110
export const WALK_CYCLE = [0, 2, 1, 3] as const satisfies readonly ActorPoseFrame[]

export function selectActorPose(
  now: number, actorId: number, moving: boolean, animated: boolean,
): ActorPoseFrame {
  if (!animated || !moving) return 0
  return WALK_CYCLE[(Math.floor(now / WALK_FRAME_DURATION_MS) + actorId) % WALK_CYCLE.length]
}
