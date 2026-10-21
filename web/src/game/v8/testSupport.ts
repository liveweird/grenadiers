import type { GameState, RoomPhase, RoomView, Seat, Snapshot } from './schema'
import { validateGameplayV8ServerMessage } from './validation'

const roomId = 'b'.repeat(32)

function room(revision: number, phase: RoomPhase, roundId: number): RoomView {
  return {
    roomId, roomCode: 'ABCDEFGH', roundId, revision, tick: revision,
    rulesVersion: 5, mapId: 'generated_v2', roundNumber: 1,
    scores: { one: 0, two: 0 }, lastRound: null,
    settings: {
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
      enabledItems: ['heart'],
    },
    availableItems: [
      'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield',
      'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot',
      'shortFlame', 'longFuse', 'reverseControls',
    ],
    settingsRevision: 1, settingsEditor: 'one',
    seats: [
      { seat: 'one', occupied: true, ready: false, rematchReady: false,
        connection: { type: 'connected' } },
      { seat: 'two', occupied: true, ready: false, rematchReady: false,
        connection: { type: 'connected' } },
    ],
    phase,
  }
}

function state(): GameState {
  return {
    runningUpdates: 1, remainingTicks: 10_799, physicalTick: 1,
    arena: {
      width: 15, height: 13,
      terrain: Array.from({ length: 195 }, () => 'floor' as const), blocks: [],
    },
    actors: [
      {
        type: 'visible', id: 1, seat: 'one', position: { x: 512, y: 512 }, heading: 'east',
        speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        inventory: { equipment: null, weapon: null }, effects: [],
        damageGraceRemainingTicks: 0,
      },
      {
        type: 'visible', id: 2, seat: 'two', position: { x: 14_848, y: 12_800 }, heading: 'west',
        speedLevel: 0, alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        inventory: { equipment: null, weapon: null }, effects: [],
        damageGraceRemainingTicks: 0,
      },
    ],
    bombs: [], flames: [], pickups: [],
  }
}

export function snapshot(revision: number, roundId = 1): Snapshot {
  return {
    type: 'snapshot', room: room(revision, { type: 'running' }, roundId),
    state: state(), events: [],
  }
}

export function validateRecipient(value: unknown, ownSeat: Seat): boolean {
  const result = validateGameplayV8ServerMessage(value)
  if (!result.valid || (result.value.type !== 'snapshot' && result.value.type !== 'admitted')) {
    return false
  }
  const state = result.value.state
  if (!state) return true
  const cloakEnabled = result.value.room.settings.enabledItems.includes('invisibilityCloak')
  const unique = (values: readonly unknown[]): boolean => new Set(values).size === values.length
  if (!unique(state.actors.map((actor) => actor.id))
    || !unique(state.actors.map((actor) => actor.seat))) return false
  return state.actors.some((actor) => actor.seat === ownSeat && actor.type === 'visible')
    && state.actors.every((actor) => {
      if (actor.seat === ownSeat) return actor.type === 'visible'
      if (actor.type === 'hidden') return cloakEnabled && actor.alive
      return !actor.effects.some((effect) => effect.kind === 'invisibilityCloak')
    })
}

