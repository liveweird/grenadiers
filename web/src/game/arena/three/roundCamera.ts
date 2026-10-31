import type { OrthographicCamera } from 'three'
import type { SnapshotMessage } from '../types'
import { applyArenaCameraOffset } from './camera'

export const ROUND_CAMERA_DURATION_MS = 900
const MAX_OBSERVATION_GAP_MS = 1_000
const YAW_RADIANS = 7 * Math.PI / 180

export type RoundCameraVariant = 'zoomIn' | 'zoomOut' | 'rotateLeft' | 'rotateRight'
type EffectKind = 'opening' | 'ending'
interface ActiveEffect { variant: RoundCameraVariant; kind: EffectKind; startedAt: number }
interface Observation {
  admission: string
  roundId: number
  phase: string
  tick: number
  receivedAt: number
}
export interface RoundCameraSample { zoom: number; yaw: number }

const VARIANTS: readonly RoundCameraVariant[] = [
  'zoomIn', 'zoomOut', 'rotateLeft', 'rotateRight',
]
const neutral = (): RoundCameraSample => ({ zoom: 1, yaw: 0 })
const smoothstep = (value: number): number => value * value * (3 - 2 * value)
const snapshotTick = (snapshot: SnapshotMessage): number =>
  snapshot.state.physicalTick ?? snapshot.state.runningUpdates ?? 0
const admissionIdentity = (snapshot: SnapshotMessage): string =>
  `${snapshot.room.roomId}:${snapshot.room.presentationBaseline ?? 'legacy'}`
const freshOpening = (
  opening: SnapshotMessage['presentationOpening'], phase: string, receivedAt: number,
  seen: string | null,
): boolean => Boolean(opening && opening.key !== seen && phase === 'running'
  && receivedAt >= opening.bornAt && receivedAt - opening.bornAt < ROUND_CAMERA_DURATION_MS)

function variantOffset(
  variant: RoundCameraVariant, kind: EffectKind, strength: number,
): RoundCameraSample {
  if (variant === 'zoomIn') {
    return { zoom: 1 + (kind === 'opening' ? -0.09 : 0.075) * strength, yaw: 0 }
  }
  if (variant === 'zoomOut') {
    return { zoom: 1 + (kind === 'opening' ? 0.075 : -0.09) * strength, yaw: 0 }
  }
  if (variant === 'rotateLeft') return { zoom: 1, yaw: -YAW_RADIANS * strength }
  return { zoom: 1, yaw: YAW_RADIANS * strength }
}

function sampleRoundCameraEffect(
  effect: ActiveEffect | null, now: number,
): RoundCameraSample {
  if (!effect) return neutral()
  const progress = Math.max(0, Math.min(1, (now - effect.startedAt) / ROUND_CAMERA_DURATION_MS))
  if (progress >= 1) return neutral()
  const strength = effect.kind === 'opening'
    ? 1 - smoothstep(progress)
    : Math.sin(Math.PI * progress) ** 2
  return variantOffset(effect.variant, effect.kind, strength)
}

export interface RoundCameraController {
  update(snapshot: SnapshotMessage, receivedAt: number): RoundCameraVariant | null
  apply(camera: OrthographicCamera, board: { width: number; height: number }, now: number): void
  reapply(camera: OrthographicCamera, board: { width: number; height: number }): void
  clear(camera?: OrthographicCamera, board?: { width: number; height: number }): void
  setReducedMotion(reduced: boolean, camera: OrthographicCamera,
    board: { width: number; height: number }): void
}

/** Tracks only observed lifecycle boundaries; it never predicts authoritative progress. */
export function createRoundCameraController(
  random: () => number = Math.random,
): RoundCameraController {
  let previous: Observation | null = null
  let effect: ActiveEffect | null = null
  let lastSample = neutral()
  let reducedMotion = false
  let seenOpening: string | null = null

  const pick = (kind: EffectKind, now: number): RoundCameraVariant => {
    const index = Math.min(VARIANTS.length - 1, Math.max(0, Math.floor(random() * VARIANTS.length)))
    const variant = VARIANTS[index]!
    effect = { variant, kind, startedAt: now }
    return variant
  }
  const reset = (): void => {
    effect = null
    lastSample = neutral()
  }
  const controller: RoundCameraController = {
    update(snapshot, receivedAt) {
      const current: Observation = {
        admission: admissionIdentity(snapshot),
        roundId: snapshot.room.roundId,
        phase: snapshot.room.phase.type,
        tick: snapshotTick(snapshot),
        receivedAt,
      }
      const old = previous
      previous = current
      const opening = snapshot.presentationOpening
      const validOpening = freshOpening(opening, current.phase, receivedAt, seenOpening)
      const openingBornAt = opening?.bornAt ?? receivedAt
      const consumeOpening = (): void => { seenOpening = opening?.key ?? seenOpening }
      if (reducedMotion) {
        consumeOpening()
        reset()
        return null
      }
      if (!old) {
        consumeOpening()
        return validOpening ? pick('opening', openingBornAt) : null
      }
      const sameAdmission = old.admission === current.admission
      const sameRound = sameAdmission && old.roundId === current.roundId
      const timely = receivedAt >= old.receivedAt
        && receivedAt - old.receivedAt <= MAX_OBSERVATION_GAP_MS
      const forward = current.tick >= old.tick
      if (!sameAdmission || !timely || (sameRound && !forward)) {
        consumeOpening()
        reset()
        return null
      }
      if (current.phase === 'running') {
        consumeOpening()
        if (validOpening) return pick('opening', openingBornAt)
        if (old.phase !== 'running') reset()
        return null
      }
      consumeOpening()
      const terminal = current.phase === 'result' || current.phase === 'matchResult'
      if (sameRound && old.phase === 'running' && terminal) return pick('ending', receivedAt)
      return null
    },
    apply(camera, board, now) {
      const sampled = reducedMotion ? neutral() : sampleRoundCameraEffect(effect, now)
      if (effect && now - effect.startedAt >= ROUND_CAMERA_DURATION_MS) effect = null
      lastSample = sampled
      applyArenaCameraOffset(camera, board, sampled)
    },
    reapply(camera, board) {
      applyArenaCameraOffset(camera, board, lastSample)
    },
    clear(camera, board) {
      previous = null
      reset()
      if (camera && board) applyArenaCameraOffset(camera, board, lastSample)
    },
    setReducedMotion(reduced, camera, board) {
      if (reducedMotion === reduced) return
      reducedMotion = reduced
      previous = null
      reset()
      applyArenaCameraOffset(camera, board, lastSample)
    },
  }
  return controller
}
