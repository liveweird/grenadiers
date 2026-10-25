export type FreezeActor = {
  type: 'visible'
  alive: boolean
  voluntaryActionsFrozen: boolean
  effects: readonly { kind: string }[]
} | {
  type: 'hidden'
  alive: boolean
}

export function freezeSemanticError(
  actors: readonly FreezeActor[], stopwatchEnabled: boolean,
): boolean {
  for (const [index, actor] of actors.entries()) {
    if (actor.type !== 'visible') continue
    if ((!actor.alive || !stopwatchEnabled) && actor.voluntaryActionsFrozen) return true
    const opponent = actors[1 - index]
    if (opponent?.type === 'visible') {
      const expected = actor.alive && opponent.alive
        && opponent.effects.some((effect) => effect.kind === 'stopwatch')
      if (actor.voluntaryActionsFrozen !== expected) return true
    }
  }
  return false
}
