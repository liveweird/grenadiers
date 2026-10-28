import type { BombDrawing } from '../src/game/arena/entityArt'
import type { ArenaScene } from '../src/game/arena/scene'
import type { SnapshotMessage } from '../src/game/arena/types'
import { diagnosticSnapshot } from './snapshot'

function assertBombState(bomb: BombDrawing | undefined, stage: string, expected: {
  scale: number; spark: boolean; pause: boolean; remote?: boolean; contact?: boolean
}): void {
  const stableNodes = bomb?.root.visible && bomb.root.scale.x === 1 && bomb.shadow.visible && bomb.shadow.scale.x === 1 && bomb.shell.visible
  if (!bomb || !stableNodes || bomb.shell.scale.x !== expected.scale
    || bomb.spark.visible !== expected.spark || bomb.pause.visible !== expected.pause
    || bomb.remote !== (expected.remote ?? false) || bomb.contact !== (expected.contact ?? false)) {
    throw new Error(`Invalid normalized bomb state at ${stage}`)
  }
}

export function representativeSnapshot(): SnapshotMessage {
  const snapshot = diagnosticSnapshot(1)
  snapshot.state.pickups = [
    { cell: { column: 2, row: 3 }, kind: 'shield' },
    { cell: { column: 4, row: 3 }, kind: 'remoteBomb' },
    { cell: { column: 6, row: 3 }, kind: 'slowFoot' },
  ]
  snapshot.state.bombs = [{
    id: 31, owner: 10, range: 4, placedAt: 1, cell: { column: 5, row: 5 },
    blast: 'power', trigger: { type: 'timed', remainingTicks: 100, paused: false },
  }]
  snapshot.state.flames = [{ cell: { column: 6, row: 5 }, expiresAt: 20 }]
  snapshot.state.oilBarrels = [{ cell: { column: 8, row: 5 }, state: 'burning' }]
  Object.assign(snapshot.state.actors[0], {
    damageGraceRemainingTicks: 30, shieldRemainingTicks: 40,
    goldenFlameRemainingTicks: 50, shortFuseRemainingTicks: 60,
    slowFootRemainingTicks: [70, 80], kickEquipped: true,
  })
  return snapshot
}

export function updateScenes(
  scenes: readonly ArenaScene[], snapshot: SnapshotMessage, now: number,
): void {
  scenes.forEach((scene) => scene.update(snapshot, now))
}

export function renderScenes(scenes: readonly ArenaScene[], now: number): void {
  scenes.forEach((scene) => scene.render(now))
}

export function setReducedMotion(scenes: readonly ArenaScene[], reduced: boolean): void {
  scenes.forEach((scene) => scene.setReducedMotion(reduced))
}

export function captureBombStages(
  scenes: readonly ArenaScene[], getBomb: () => BombDrawing | undefined,
  prime: () => void, capture: (stage: string) => void,
): void {
  setReducedMotion(scenes, false)
  for (const remainingTicks of [180, 165, 120, 112, 60, 57, 3]) {
    const snapshot = representativeSnapshot()
    const stage = `bomb-fuse-${remainingTicks}`
    snapshot.room.roundId = 100 + remainingTicks
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks, paused: false }
    updateScenes(scenes, snapshot, 0)
    prime()
    capture(stage)
    const pulsed = [180, 120, 60].includes(remainingTicks)
    assertBombState(getBomb(), stage, { scale: pulsed ? 1.125 : 1, spark: pulsed, pause: false })
  }
  for (const [index, type] of (['paused', 'remote', 'contact'] as const).entries()) {
    const snapshot = representativeSnapshot()
    const stage = `bomb-${type}`
    snapshot.room.roundId = 300 + index
    snapshot.state.bombs[0].id = 300 + index
    snapshot.state.bombs[0].trigger = type === 'paused'
      ? { type: 'timed', remainingTicks: 60, paused: true } : { type }
    updateScenes(scenes, snapshot, 0)
    prime()
    capture(stage)
    assertBombState(getBomb(), stage, { scale: 1, spark: false,
      pause: type === 'paused', remote: type === 'remote', contact: type === 'contact' })
  }
  for (const [index, blast] of (['standard', 'piercing', 'power'] as const).entries()) {
    const snapshot = representativeSnapshot()
    const stage = `bomb-pulse-${blast}`
    snapshot.room.roundId = 400 + index
    snapshot.state.bombs[0].blast = blast
    snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: false }
    updateScenes(scenes, snapshot, 0)
    prime()
    capture(stage)
    assertBombState(getBomb(), stage, { scale: 1.125, spark: true, pause: false })
  }
  const snapshot = representativeSnapshot()
  snapshot.room.roundId = 500
  snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: true }
  updateScenes(scenes, snapshot, 0)
  prime()
  capture('bomb-animation-paused')
  assertBombState(getBomb(), 'bomb-animation-paused', { scale: 1, spark: false, pause: true })
  snapshot.state.bombs[0].trigger = { type: 'timed', remainingTicks: 60, paused: false }
  updateScenes(scenes, snapshot, 1)
  prime()
  capture('bomb-animation-resumed')
  assertBombState(getBomb(), 'bomb-animation-resumed', { scale: 1.125, spark: true, pause: false })
  renderScenes(scenes, 10_000)
  capture('bomb-animation-stale-render')
  assertBombState(getBomb(), 'bomb-animation-stale-render', { scale: 1.125, spark: true, pause: false })
  setReducedMotion(scenes, true)
  renderScenes(scenes, 10_001)
  capture('bomb-animation-reduced-motion')
  assertBombState(getBomb(), 'bomb-animation-reduced-motion', { scale: 1, spark: true, pause: false })
  setReducedMotion(scenes, false)
  snapshot.state.physicalTick = Number(snapshot.state.physicalTick) + 1
  snapshot.state.bombs[0].cell.column += 1
  updateScenes(scenes, snapshot, 2)
  capture('bomb-animation-moved-same-id')
  assertBombState(getBomb(), 'bomb-animation-moved-same-id', { scale: 1.125, spark: true, pause: false })
  snapshot.room.phase = { type: 'result' }
  updateScenes(scenes, snapshot, 3)
  capture('bomb-animation-terminal')
  assertBombState(getBomb(), 'bomb-animation-terminal', { scale: 1, spark: true, pause: false })
}

