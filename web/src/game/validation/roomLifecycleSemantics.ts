type Connection = { type: string; expiresAt?: number }
type SeatStatus = {
  seat: string
  occupied: boolean
  ready: boolean
  rematchReady: boolean
  controller: { type: string }
  connection: Connection
}
type RoundResult = { type: string; seat?: string }
type CompletedRound = {
  roundId: number
  roundNumber: number
  endedAt: number
  result: RoundResult
}
type MatchResult = { type: string; seat?: string; reason?: string }
type Phase = {
  type: string
  kind?: string
  endsAt?: number
  holdUntil?: number
  endedAt?: number
  rematchAt?: number
  result?: MatchResult
}
type LifecycleRoom = {
  tick: number
  roundId: number
  roundNumber: number
  scores: readonly { seat: string; wins: number }[]
  lastRound: CompletedRound | null
  seats: readonly SeatStatus[]
  phase: Phase
  settings: { winsToFinish: number }
}

const connected = ({ type }: Connection): boolean =>
  type === 'connected' || type === 'serverControlled'
const anyReady = (room: LifecycleRoom): boolean =>
  room.seats.some((seat) => seat.ready || seat.rematchReady)
const anyRematch = (room: LifecycleRoom): boolean =>
  room.seats.some((seat) => seat.rematchReady)
const allOccupied = (room: LifecycleRoom): boolean => room.seats.every((seat) => seat.occupied)
const score = (room: LifecycleRoom, seat: string): number | undefined =>
  room.scores.find((value) => value.seat === seat)?.wins
const scoresBelow = (room: LifecycleRoom): boolean =>
  room.scores.every(({ wins }) => wins < room.settings.winsToFinish)
const futureWithin = (now: number, future: number | undefined, maximum: number): boolean =>
  future !== undefined && future > now && future - now <= maximum

function readinessConsensus(room: LifecycleRoom, rematch: boolean): boolean {
  let eligible = 0
  let connectedHuman = false
  const complete = room.seats.every((seat) => {
    if (connected(seat.connection)) {
      eligible += 1
      connectedHuman ||= seat.controller.type === 'human'
      return rematch ? seat.rematchReady : seat.ready
    }
    return seat.connection.type === 'expired'
  })
  return complete && eligible >= 2 && connectedHuman
}

function seatsInvalid(room: LifecycleRoom): boolean {
  const initialReadiness = room.phase.type === 'waiting'
    || (room.phase.type === 'countdown' && room.phase.kind === 'initial')
  const rematchAt = room.phase.type === 'matchResult' ? room.phase.rematchAt : undefined
  return room.seats.some((seat) => {
    const cpu = seat.controller.type === 'cpu'
    const cpuReady = cpu && (room.phase.type === 'waiting' || room.phase.type === 'countdown')
    const cpuRematch = cpu && rematchAt !== undefined && room.tick >= rematchAt
    const invalidReady = cpu
      ? seat.ready !== cpuReady || seat.rematchReady !== cpuRematch
      : (seat.ready && (!connected(seat.connection) || !initialReadiness))
        || (seat.rematchReady && (!connected(seat.connection)
          || rematchAt === undefined || room.tick < rematchAt))
    const invalidDisconnect = seat.connection.type === 'disconnected'
      && (!futureWithin(room.tick, seat.connection.expiresAt, 600)
        || (room.phase.type === 'matchResult'
          && seat.connection.expiresAt! - room.phase.endedAt! > 600))
    return invalidReady || invalidDisconnect
  }) || (room.phase.type === 'matchResult' && readinessConsensus(room, true))
}

function historyInvalid(room: LifecycleRoom): boolean {
  const total = room.scores.reduce((sum, value) => sum + value.wins, 0)
  const last = room.lastRound
  if (!last) return total !== 0
  if (last.endedAt > room.tick || last.roundNumber > room.roundNumber
    || last.roundId > room.roundId || last.roundId < last.roundNumber
    || total > last.roundNumber) return true
  return (last.result.type === 'winner' && score(room, last.result.seat!) === 0)
    || (last.result.type === 'draw' && total >= last.roundNumber)
}

function nonfinalReservations(room: LifecycleRoom): boolean {
  return allOccupied(room)
    && room.seats.filter(({ connection }) => connection.type !== 'expired').length >= 2
}

function countdownParticipants(room: LifecycleRoom): boolean {
  const eligible = room.seats.filter(({ connection }) => connected(connection)).length
  return eligible >= 2
    && room.seats.some((seat) => seat.controller.type === 'human' && connected(seat.connection))
    && room.seats.every(({ connection }) => connected(connection) || connection.type === 'expired')
}

function scoreWinnerInvalid(room: LifecycleRoom): boolean {
  const { phase } = room
  const endedAt = phase.endedAt!
  const result = phase.result!
  const last = room.lastRound
  return score(room, result.seat!) !== room.settings.winsToFinish
    || room.scores.some((value) => value.seat !== result.seat
      && value.wins >= room.settings.winsToFinish)
    || last === null || last.roundNumber !== room.roundNumber || last.roundId !== room.roundId
    || last.endedAt !== endedAt || last.result.type !== 'winner'
    || last.result.seat !== result.seat
}

function nonscoreTerminalInvalid(room: LifecycleRoom): boolean {
  const result = room.phase.result!
  if (!scoresBelow(room)) return true
  if (result.type === 'winner' && result.reason === 'forfeit'
    && room.seats.some((seat) => seat.seat !== result.seat
      && seat.connection.type !== 'vacant' && seat.connection.type !== 'expired')) return true
  if (result.type === 'abandoned' && room.seats.some((seat) =>
    seat.controller.type === 'human'
      && seat.connection.type !== 'vacant' && seat.connection.type !== 'expired')) return true
  const last = room.lastRound
  return (last !== null || room.roundNumber !== 1)
    && last?.roundNumber !== room.roundNumber
    && (last === null || last.roundNumber + 1 !== room.roundNumber || last.roundId >= room.roundId)
}

function terminalInvalid(room: LifecycleRoom): boolean {
  const { phase } = room
  const endedAt = phase.endedAt!
  const rematchAt = phase.rematchAt!
  if (room.roundNumber === 0 || endedAt > room.tick || rematchAt !== endedAt + 180
    || room.seats.some((seat) => seat.ready)
    || (room.tick < rematchAt && anyRematch(room))
    || (allOccupied(room) && room.seats.every((seat) => seat.rematchReady))
    || (room.lastRound?.endedAt ?? 0) > endedAt) return true
  return phase.result?.type === 'winner' && phase.result.reason === 'roundWins'
    ? scoreWinnerInvalid(room) : nonscoreTerminalInvalid(room)
}

function waitingInvalid(room: LifecycleRoom): boolean {
  return room.roundNumber !== 0 || room.scores.some(({ wins }) => wins !== 0)
    || room.lastRound !== null || readinessConsensus(room, false) || anyRematch(room)
}

function initialInvalid(room: LifecycleRoom): boolean {
  return room.roundNumber !== 0 || room.scores.some(({ wins }) => wins !== 0)
    || room.lastRound !== null || !readinessConsensus(room, false)
    || !futureWithin(room.tick, room.phase.endsAt, 180) || anyRematch(room)
}

function interRoundInvalid(room: LifecycleRoom): boolean {
  const last = room.lastRound
  return last === null || room.roundNumber === 0 || !countdownParticipants(room)
    || room.seats.some((seat) => seat.controller.type === 'human' && seat.ready)
    || last.roundNumber !== room.roundNumber || last.roundId >= room.roundId
    || room.tick < last.endedAt + 180 || !futureWithin(room.tick, room.phase.endsAt, 180)
    || !scoresBelow(room)
}

function runningInvalid(room: LifecycleRoom): boolean {
  if (room.roundNumber === 0 || !nonfinalReservations(room)
    || anyReady(room) || !scoresBelow(room)) return true
  const last = room.lastRound
  if (last === null) {
    return room.roundNumber !== 1 || room.scores.some(({ wins }) => wins !== 0)
  }
  return last.roundNumber + 1 !== room.roundNumber || last.roundId >= room.roundId
}

function resultInvalid(room: LifecycleRoom): boolean {
  const last = room.lastRound
  return last === null || room.roundNumber === 0 || !nonfinalReservations(room)
    || anyReady(room) || !scoresBelow(room) || last.roundNumber !== room.roundNumber
    || last.roundId > room.roundId || room.phase.holdUntil !== last.endedAt + 180
}

function phaseInvalid(room: LifecycleRoom): boolean {
  const { phase } = room
  if (phase.type === 'waiting') return waitingInvalid(room)
  if (phase.type === 'countdown') {
    return phase.kind === 'initial' ? initialInvalid(room) : interRoundInvalid(room)
  }
  if (phase.type === 'running') return runningInvalid(room)
  if (phase.type === 'result') return resultInvalid(room)
  if (phase.type === 'matchResult') return terminalInvalid(room)
  return phase.endedAt !== room.tick || room.roundNumber !== 0
    || room.scores.some(({ wins }) => wins !== 0) || room.lastRound !== null || anyReady(room)
}

export function roomLifecycleSemanticError(room: LifecycleRoom): 'invalidLifecycle' | null {
  const seats = new Set(room.seats.map(({ seat }) => seat))
  const winnerOutsideRoom = room.lastRound?.result.type === 'winner'
    && !seats.has(room.lastRound.result.seat!)
    || room.phase.type === 'matchResult' && room.phase.result?.type === 'winner'
      && !seats.has(room.phase.result.seat!)
  return room.roundId < room.roundNumber
    || room.scores.some(({ wins }) => wins > room.settings.winsToFinish)
    || winnerOutsideRoom || seatsInvalid(room) || historyInvalid(room) || phaseInvalid(room)
    ? 'invalidLifecycle' : null
}
