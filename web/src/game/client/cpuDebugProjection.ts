import type { GameClientState } from './clientTypes'
import type { CpuDebugResponse, CpuDebugRow } from './cpuDebugTypes'
import type { Cell, Seat } from '../v20/schema'

export interface ProjectedCpuDebugRow {
  seat: Seat
  intent: CpuDebugRow | null
}

function sameCell(left: Cell, right: Cell): boolean {
  return left.column === right.column && left.row === right.row
}

function cellIsInArena(cell: Cell, state: GameClientState): boolean {
  const arena = state.snapshot?.state.arena
  return Boolean(arena && cell.column < arena.width && cell.row < arena.height)
}

function targetIsVisible(row: Extract<CpuDebugRow, { status: 'available' }>, state: GameClientState): boolean {
  const target = row.target
  const snapshot = state.snapshot?.state
  if (!target || !snapshot) return target === null
  if (target.type === 'pickup') {
    return snapshot.pickups.some((pickup) => sameCell(pickup.cell, target.cell) && pickup.kind === target.kind)
  }
  if (target.type === 'block') return snapshot.arena.blocks.some((cell) => sameCell(cell, target.cell))
  if (target.type === 'opponent') {
    return snapshot.actors.some((actor) => actor.seat === target.seat && actor.type === 'visible')
  }
  return cellIsInArena(target.cell, state)
}

function recipientIsBombBlind(state: GameClientState): boolean {
  const own = state.snapshot?.state.actors.find((actor) => actor.seat === state.youSeat)
  return own?.type === 'visible' && own.effects.some((effect) => effect.kind === 'invisibleBombs')
}

export function cpuDebugResponseMatchesState(response: CpuDebugResponse, state: GameClientState): boolean {
  const room = state.room
  if (!room || response.roomId !== room.roomId || response.roundId !== room.roundId
    || response.youSeat !== state.youSeat) return false
  const cpuSeats = new Set(room.seats.filter(({ controller }) => controller.type === 'cpu').map(({ seat }) => seat))
  const responseSeats = response.cpus.map(({ seat }) => seat)
  return new Set(responseSeats).size === responseSeats.length
    && responseSeats.every((seat) => cpuSeats.has(seat))
    && response.cpus.every((row) => row.status === 'unavailable' || row.observedTick <= response.tick)
}

export function projectCpuDebug(response: CpuDebugResponse | null, state: GameClientState): ProjectedCpuDebugRow[] {
  const cpuSeats = state.room?.seats
    .filter(({ controller }) => controller.type === 'cpu').map(({ seat }) => seat) ?? []
  const unavailable = cpuSeats.map((seat) => ({ seat, intent: null }))
  if (!response || !cpuDebugResponseMatchesState(response, state) || recipientIsBombBlind(state)) return unavailable
  const snapshot = state.snapshot?.state
  if (!snapshot) return unavailable
  return cpuSeats.map((seat) => {
    const row = response.cpus.find((candidate) => candidate.seat === seat)
    const actorVisible = snapshot.actors.some((actor) => actor.seat === seat && actor.type === 'visible')
    const approachVisible = row?.status !== 'available' || row.approach === null || cellIsInArena(row.approach, state)
    const visible = row?.status === 'available' && actorVisible && approachVisible && targetIsVisible(row, state)
    return { seat, intent: visible ? row : null }
  })
}
