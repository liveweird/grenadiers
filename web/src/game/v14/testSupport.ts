import type { GameState, RoomView, Snapshot } from './schema'

const roomId = 'b'.repeat(32)

function room(revision: number): RoomView {
  return {
    roomId, roomCode: 'ABCDEFGH', roundId: 1, revision, tick: revision,
    configuration: {
      controllers: ['human', 'human'], cpuDifficulty: null, board: { width: 15, height: 13 },
    },
    rulesVersion: 5, mapId: 'generated_v2', roundNumber: 1,
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
    phase: { type: 'running' },
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

export function snapshot(revision: number): Snapshot {
  return { type: 'snapshot', room: room(revision), state: gameState(), events: [] }
}
