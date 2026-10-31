import type {
  DeathAppearance, RenderablePickupKind, SnapshotMessage,
} from '../src/game/arena/types'
import { RENDERABLE_PICKUP_KINDS } from '../src/game/arena/types'
import { gameSnapshot } from '../src/test/game-snapshot'

export type BoardName = 'small' | 'medium' | 'large'
export type SceneName =
  | 'empty' | 'actorBomb' | 'showcase' | 'teleportA' | 'teleportB' | 'newRoom'
  | 'actorOnly' | 'remoteActor' | 'lairOnly'
export type DeathOverlay = 'none' | 'flame' | 'burningOil'
export type BloodLayerMode = 'subjectOnly' | 'poolOnly' | 'combined'

const BOARDS = {
  small: { width: 11 as const, height: 9 as const },
  medium: { width: 13 as const, height: 11 as const },
  large: { width: 15 as const, height: 13 as const },
}

function arena(name: BoardName): SnapshotMessage['state']['arena'] {
  const { width, height } = BOARDS[name]
  return {
    width,
    height,
    terrain: Array.from({ length: width * height }, (_value, index) => {
      const column = index % width
      const row = Math.floor(index / width)
      return column === 0 || column === width - 1 || row === 0 || row === height - 1
        || (column % 2 === 0 && row % 2 === 0) ? 'wall' : 'floor'
    }),
    blocks: [{ column: 3, row: 3 }, { column: width - 4, row: height - 4 }],
  }
}

function base(board: BoardName, roomId = 'three-arena-static-room'): SnapshotMessage {
  const snapshot = structuredClone(gameSnapshot())
  snapshot.room = {
    roomId,
    roundId: 1,
    phase: { type: 'waiting' },
    presentationBaseline: 'three-arena-static-v1',
  }
  snapshot.state.physicalTick = 1
  snapshot.state.arena = arena(board)
  snapshot.state.actors = []
  snapshot.state.bombs = []
  snapshot.state.flames = []
  snapshot.state.pickups = []
  snapshot.state.oilBarrels = []
  snapshot.state.stormCloud = null
  snapshot.state.lairs = []
  snapshot.state.mobs = []
  return snapshot
}

function actor(
  id: number,
  seat: 'one' | 'two' | 'three' | 'four',
  column: number,
  row: number,
): SnapshotMessage['state']['actors'][number] {
  return {
    id, seat, position: { x: (column + 0.5) * 1024, y: (row + 0.5) * 1024 },
    heading: seat === 'one' ? 'east' : seat === 'two' ? 'south' : seat === 'three' ? 'west' : 'north',
    alive: true, lastTeleportTick: null,
  }
}

function actorBombScene(snapshot: SnapshotMessage): void {
  snapshot.state.actors = [actor(10, 'one', 1, 1)]
  snapshot.state.bombs = [{
    id: 30, cell: { column: 3, row: 1 }, owner: 10, range: 2, placedAt: 1,
    trigger: { type: 'timed', remainingTicks: 180, paused: true }, blast: 'standard',
  }]
}

function pickupCells(width: number, height: number): Array<{ column: number; row: number }> {
  const cells: Array<{ column: number; row: number }> = []
  for (let row = 1; row < height - 1; row += 1) {
    for (let column = 1; column < width - 1; column += 1) {
      if (column % 2 === 0 && row % 2 === 0) continue
      cells.push({ column, row })
    }
  }
  return cells
}

function showcase(snapshot: SnapshotMessage): void {
  const { width, height } = snapshot.state.arena
  const cells = pickupCells(width, height)
  snapshot.state.pickups = RENDERABLE_PICKUP_KINDS.map((kind: RenderablePickupKind, index) => ({
    cell: cells[index], kind,
  }))
  snapshot.state.actors = [
    { ...actor(10, 'one', 1, height - 2), kickEquipped: true, shieldRemainingTicks: 60 },
    { ...actor(20, 'two', width - 2, 1), punchEquipped: true, stopwatchRemainingTicks: 60 },
    { ...actor(30, 'three', width - 2, height - 2), braceletEquipped: true, piercingArmed: true },
    { ...actor(40, 'four', Math.floor(width / 2), height - 2), wallPassEquipped: true, powerArmed: true },
  ]
  snapshot.state.bombs = [
    { id: 1, cell: { column: 1, row: 1 }, owner: 10, range: 2, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 160, paused: true }, blast: 'standard' },
    { id: 2, cell: { column: 3, row: 1 }, owner: 20, range: 3, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 80, paused: true }, blast: 'piercing' },
    { id: 3, cell: { column: 5, row: 1 }, owner: 30, range: 4, placedAt: 1,
      trigger: { type: 'remote' }, blast: 'power' },
    { id: 4, cell: { column: 7, row: 1 }, owner: 40, range: 2, placedAt: 1,
      trigger: { type: 'contact' }, blast: 'standard' },
  ]
  snapshot.state.flames = [{ cell: { column: 1, row: 3 }, expiresAt: 999 }]
  snapshot.state.oilBarrels = [
    { cell: { column: 3, row: 3 }, state: 'unlit' },
    { cell: { column: 5, row: 3 }, state: 'burning' },
  ]
  snapshot.state.stormCloud = {
    position: { x: (width / 2) * 1024, y: (height / 2) * 1024 },
    direction: 'east', legRemainingTicks: 20,
  }
  snapshot.state.lairs = [{
    id: 90, cell: { column: width - 2, row: Math.floor(height / 2) },
    spawnRemainingTicks: 30, livingMobs: 2,
  }]
  snapshot.state.mobs = [
    { id: 91, lairId: 90, position: { x: 2.5 * 1024, y: 5.5 * 1024 }, heading: 'east', frozen: true },
    { id: 92, lairId: 90, position: { x: 4.5 * 1024, y: 5.5 * 1024 }, heading: 'west', frozen: false },
  ]
}

export function threeArenaSnapshot(board: BoardName, scene: SceneName): SnapshotMessage {
  const snapshot = base(board, scene === 'newRoom' ? 'three-arena-new-room' : undefined)
  if (scene === 'actorBomb') actorBombScene(snapshot)
  if (scene === 'showcase') showcase(snapshot)
  if (scene === 'actorOnly' || scene === 'remoteActor') {
    snapshot.state.actors = [{
      ...actor(10, 'one', 7, 6),
      ...(scene === 'remoteActor' ? { remoteArmed: true } : {}),
    }]
  }
  if (scene === 'lairOnly') {
    snapshot.state.lairs = [{
      id: 90, cell: { column: 7, row: 6 }, spawnRemainingTicks: 30, livingMobs: 0,
    }]
  }
  if (scene === 'teleportA' || scene === 'teleportB') {
    snapshot.state.physicalTick = scene === 'teleportA' ? 10 : 11
    snapshot.state.actors = [actor(10, 'one', scene === 'teleportA' ? 1 : snapshot.state.arena.width - 2,
      scene === 'teleportA' ? 1 : snapshot.state.arena.height - 2)]
    snapshot.state.actors[0].lastTeleportTick = scene === 'teleportA' ? null : 11
  }
  for (const { cell } of snapshot.state.lairs) {
    snapshot.state.arena.terrain[cell.row * snapshot.state.arena.width + cell.column] = 'wall'
  }
  return snapshot
}

export function threeArenaDeathSnapshot(
  board: BoardName,
  appearance: DeathAppearance,
  bornAt: number,
  index: number,
): SnapshotMessage {
  const snapshot = base(board)
  const cells = [
    { column: 1, row: 2 },
    { column: 3, row: 2 },
    { column: 5, row: 2 },
    { column: 7, row: 2 },
    { column: 9, row: 2 },
  ]
  const cell = cells[index % cells.length]
  snapshot.presentationDeaths = {
    epoch: index + 1,
    sequence: 1,
    cues: [{
      key: `diagnostic:${appearance}:${index}`,
      appearance,
      entityId: 100 + index,
      position: { x: (cell.column + 0.5) * 1024, y: (cell.row + 0.5) * 1024 },
      heading: 'south',
      tick: 1,
      index,
      bornAt,
    }],
  }
  return snapshot
}

export function threeArenaDeathBaselineSnapshot(
  board: BoardName,
  index: number,
): SnapshotMessage {
  const snapshot = base(board)
  snapshot.presentationDeaths = { epoch: index + 1, sequence: 0, cues: [] }
  return snapshot
}

function bloodLayerCues(bornAt: number, generation: number) {
  return [
    {
      key: `diagnostic:blood:actor:${generation}`,
      appearance: 'one' as const,
      entityId: 300 + generation,
      position: { x: 3.5 * 1024, y: 2.5 * 1024 },
      heading: 'south' as const, tick: 1, index: 0, bornAt,
    },
    {
      key: `diagnostic:blood:mob:${generation}`,
      appearance: 'mob' as const,
      entityId: 400 + generation,
      position: { x: 7.5 * 1024, y: 2.5 * 1024 },
      heading: 'south' as const, tick: 1, index: 1, bornAt,
    },
  ]
}

export function threeArenaBloodLayerSnapshot(
  mode: BloodLayerMode,
  bornAt: number,
  generation: number,
): SnapshotMessage {
  const snapshot = base('large')
  if (mode !== 'subjectOnly') {
    snapshot.presentationDeaths = {
      epoch: 300 + generation, sequence: 1, cues: bloodLayerCues(bornAt, generation),
    }
  }
  if (mode !== 'poolOnly') {
    snapshot.state.actors = [actor(10, 'one', 3, 2)]
    snapshot.state.flames = [{ cell: { column: 7, row: 2 }, expiresAt: 999 }]
  }
  return snapshot
}

export function threeArenaBloodLayerBaseline(generation: number): SnapshotMessage {
  const snapshot = base('large')
  snapshot.presentationDeaths = { epoch: 300 + generation, sequence: 0, cues: [] }
  return snapshot
}

export function threeArenaDeathOverlapSnapshot(
  overlay: DeathOverlay,
  includeDeath: boolean,
  bornAt: number,
  generation: number,
): SnapshotMessage {
  const snapshot = base('large')
  const cell = { column: 1, row: 2 }
  if (overlay === 'flame') snapshot.state.flames = [{ cell, expiresAt: 999 }]
  if (overlay === 'burningOil') snapshot.state.oilBarrels = [{ cell, state: 'burning' }]
  if (includeDeath) {
    snapshot.presentationDeaths = {
      epoch: 100 + generation,
      sequence: 1,
      cues: [{
        key: `diagnostic:overlap:${generation}`,
        appearance: 'one',
        entityId: 200 + generation,
        position: { x: 1.5 * 1024, y: 2.5 * 1024 },
        heading: 'south',
        tick: 1,
        index: generation,
        bornAt,
      }],
    }
  }
  return snapshot
}
