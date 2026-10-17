import type { BombView, GameState, RoomView, VisibleActorView } from './schema'

export const roomId = 'b'.repeat(32)

const settings = {
  startingSpeedLevel: 0,
  turnAssistUnits: 256,
  winsToFinish: 3,
  bombFuseTicks: 180,
  enabledItems: ['kick', 'shield'],
} as const

export const room: RoomView = {
  roomId,
  roomCode: 'ABCDEFGH',
  roundId: 1,
  revision: 1,
  tick: 42,
  rulesVersion: 4,
  mapId: 'generated_v1',
  roundNumber: 1,
  scores: { one: 0, two: 0 },
  lastRound: null,
  seats: [
    { seat: 'one', occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
    { seat: 'two', occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
  ],
  phase: { type: 'running' },
  settings: { ...settings, enabledItems: [...settings.enabledItems] },
  settingsRevision: 1,
  settingsEditor: 'one',
  availableItems: ['shield', 'kick', 'heart'],
}

export const visibleActor: VisibleActorView = {
  type: 'visible',
  id: 1,
  seat: 'one',
  position: { x: 0, y: 0 },
  heading: 'east',
  speedLevel: 1,
  alive: true,
  bombCapacity: 1,
  bombRange: 1,
  liveBombCount: 0,
  inventory: { equipment: null, weapon: null },
  effects: [],
  damageGraceRemainingTicks: 0,
}

export const bomb: BombView = {
  id: 3,
  cell: { column: 1, row: 1 },
  owner: 1,
  range: 2,
  placedAt: 40,
  trigger: { type: 'timed', remainingTicks: 180, paused: false },
  blast: 'standard',
}

export const state: GameState = {
  runningUpdates: 1,
  remainingTicks: 10_000,
  physicalTick: 42,
  arena: { width: 15, height: 13, terrain: Array.from({ length: 195 }, () => 'floor'), blocks: [] },
  actors: [visibleActor, { type: 'hidden', id: 2, seat: 'two', alive: true }],
  bombs: [bomb],
  flames: [],
  pickups: [],
}

export const snapshot = { type: 'snapshot', room, state, events: [] } as const
