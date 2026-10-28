export interface BombAnimationState {
  shellScale: number
  sparkVisible: boolean
}

const STATIC_TIMED_BOMB: BombAnimationState = { shellScale: 1, sparkVisible: true }

function pulsePeriod(remainingTicks: number): number {
  if (remainingTicks > 120) return 60
  if (remainingTicks > 60) return 30
  return 12
}

export function selectBombAnimation(
  remainingTicks: number | null,
  animated: boolean,
): BombAnimationState {
  if (!animated || remainingTicks === null) return STATIC_TIMED_BOMB
  const ticks = Math.max(0, Math.floor(remainingTicks))
  const period = pulsePeriod(ticks)
  const pulsing = ticks % period < period / 4
  return { shellScale: pulsing ? 1.125 : 1, sparkVisible: pulsing }
}
