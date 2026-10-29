import type { Admitted, GameState, RoomPhase, RoomView, Snapshot } from './schema'

const roomId = 'b'.repeat(32)
const resumeToken = 'c'.repeat(64)

export function room(
  revision: number,
  phase: RoomPhase = { type: 'waiting' },
  roundId = 1,
): RoomView {
  return {
    roomId, roomCode: 'ABCDEFGH', roundId, revision, tick: revision,
    configuration: {
      controllers: ['human', 'human'], cpuDifficulty: null, board: { width: 15, height: 13 },
    },
    rulesVersion: 5, mapId: 'generated_v2', roundNumber: phase.type === 'running' ? 1 : 0,
    scores: [{ seat: 'one', wins: 0 }, { seat: 'two', wins: 0 }], lastRound: null,
    settings: {
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
      enabledItems: ['heart'],
    },
    availableItems: [
      'kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb',
      'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame',
      'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls',
      'invisibleBombs', 'oilBarrel', 'teleporter',
    ],
    settingsRevision: 1, settingsEditor: 'one',
    seats: [
      { seat: 'one', controller: { type: 'human' }, occupied: true, ready: false,
        rematchReady: false, connection: { type: 'connected' } },
      { seat: 'two', controller: { type: 'human' }, occupied: true, ready: false,
        rematchReady: false, connection: { type: 'connected' } },
    ],
    phase,
  }
}

function gameState(): GameState {
  const actor = (id: number, seat: 'one' | 'two', x: number, y: number) => ({
    type: 'visible' as const, id, seat, position: { x, y }, heading: 'east' as const,
    speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
    inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
    voluntaryActionsFrozen: false, lastTeleportTick: null,
  })
  return {
    runningUpdates: 1, remainingTicks: 10_799, physicalTick: 1,
    arena: { width: 15, height: 13,
      terrain: Array.from({ length: 195 }, () => 'floor' as const), blocks: [] },
    actors: [actor(1, 'one', 512, 512), actor(2, 'two', 14_848, 12_800)],
    bombs: [], flames: [], pickups: [], oilBarrels: [],
  }
}

export function snapshot(revision: number, roundId = 1): Snapshot {
  return {
    type: 'snapshot', room: room(revision, { type: 'running' }, roundId), state: gameState(),
    events: [],
  }
}

export function admitted(revision = 1, lastActionId: number | null = null): Admitted {
  return {
    type: 'admitted', sequence: 1, room: room(revision, { type: 'running' }),
    youSeat: 'one', resumeToken, lastActionId, state: gameState(),
  }
}
