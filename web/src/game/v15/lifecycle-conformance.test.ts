import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v15-server-conformance.json'
import type { GameServerMessage } from './schema'
import { validateGameplayV15ServerMessage } from './validation'

type Case = { caseId: string; scenario: string; recipient: string | null; frame: string }
const cases = corpus.cases as Case[]
const frames = new Map(cases.map((entry) =>
  [entry.caseId, JSON.parse(entry.frame) as GameServerMessage]))

function roomState(caseId: string): Extract<GameServerMessage, { type: 'roomState' }> {
  const frame = structuredClone(frames.get(caseId))
  if (frame?.type !== 'roomState') throw new Error(`missing room state fixture ${caseId}`)
  return frame
}

function admitted(caseId: string): Extract<GameServerMessage, { type: 'admitted' }> {
  const frame = structuredClone(frames.get(caseId))
  if (frame?.type !== 'admitted') throw new Error(`missing admission fixture ${caseId}`)
  return frame
}

describe('RosterMatchLifecycle phase corpus', () => {
  it('covers concrete countdown, result, terminal, reconnect, and rematch facts', () => {
    expect(roomState('lifecycle-initial-countdown').room.phase).toMatchObject({
      type: 'countdown', kind: 'initial', endsAt: 180,
    })
    expect(roomState('lifecycle-inter-round').room.phase).toMatchObject({
      type: 'countdown', kind: 'interRound', endsAt: 540,
    })
    expect(roomState('lifecycle-round-result').room.phase).toMatchObject({
      type: 'result', holdUntil: 360,
    })
    expect(roomState('lifecycle-next-running').room).toMatchObject({
      roundId: 2, roundNumber: 2, phase: { type: 'running' },
      lastRound: { roundId: 1, roundNumber: 1 },
    })
    expect(roomState('lifecycle-score-winner').room.phase).toMatchObject({
      type: 'matchResult', result: { type: 'winner', reason: 'roundWins' },
    })
    expect(roomState('lifecycle-forfeit').room.phase).toMatchObject({
      type: 'matchResult', result: { type: 'winner', reason: 'forfeit' },
    })
    expect(roomState('lifecycle-interrupted').room.phase).toMatchObject({ type: 'interrupted' })
    expect(roomState('lifecycle-abandoned').room.phase).toMatchObject({
      type: 'matchResult', result: { type: 'abandoned' },
    })
    expect(roomState('lifecycle-rematch-eligible').room.seats.map((seat) => seat.rematchReady))
      .toEqual([true, true, false])
    expect(roomState('lifecycle-disconnected').room.seats[0]!.connection).toEqual({
      type: 'disconnected', expiresAt: 780,
    })
    expect(admitted('lifecycle-result-admission')).toMatchObject({
      lastActionId: 1,
      room: { phase: { type: 'result' }, lastRound: { roundId: 1, endedAt: 180 } },
      state: { physicalTick: 180 },
    })
    expect(admitted('lifecycle-forfeit-admission')).toMatchObject({
      lastActionId: 1,
      room: { phase: { type: 'matchResult', result: { reason: 'forfeit' } } },
      state: { physicalTick: 180 },
    })
    expect(admitted('lifecycle-score-winner-admission')).toMatchObject({
      lastActionId: 1,
      room: {
        phase: { type: 'matchResult', result: { type: 'winner', reason: 'roundWins' } },
        lastRound: { roundId: 3, roundNumber: 3, endedAt: 900 },
      },
      state: { physicalTick: 900 },
    })
    expect(admitted('lifecycle-continued-forfeit-admission')).toMatchObject({
      lastActionId: 1,
      room: {
        roundId: 2,
        phase: { type: 'matchResult', endedAt: 540, result: { reason: 'forfeit' } },
        lastRound: { roundId: 1, roundNumber: 1, endedAt: 180 },
      },
      state: { physicalTick: 540 },
    })
  })

  it('rejects a retained result admission action cursor after its room epoch advances', () => {
    const frame = admitted('lifecycle-result-admission')
    frame.room.roundId += 1
    expect(validateGameplayV15ServerMessage(frame)).toEqual({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })
  })

  it.each([
    ['initial readiness', 'lifecycle-initial-countdown', (frame: ReturnType<typeof roomState>) => {
      frame.room.seats[0]!.ready = false
    }],
    ['initial deadline', 'lifecycle-initial-countdown', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.phase.type === 'countdown') frame.room.phase.endsAt = frame.room.tick
    }],
    ['inter-round human readiness', 'lifecycle-inter-round', (frame: ReturnType<typeof roomState>) => {
      frame.room.seats[0]!.ready = true
    }],
    ['inter-round history', 'lifecycle-inter-round', (frame: ReturnType<typeof roomState>) => {
      frame.room.lastRound = null
    }],
    ['next-round history', 'lifecycle-next-running', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.lastRound) frame.room.lastRound.roundNumber = frame.room.roundNumber
    }],
    ['post-terminal running score', 'lifecycle-next-running', (frame: ReturnType<typeof roomState>) => {
      frame.room.roundId = 4
      frame.room.roundNumber = 4
      frame.room.scores[0]!.wins = 3
      if (frame.room.lastRound) {
        frame.room.lastRound.roundId = 3
        frame.room.lastRound.roundNumber = 3
      }
    }],
    ['result deadline', 'lifecycle-round-result', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.phase.type === 'result') frame.room.phase.holdUntil += 1
    }],
    ['draw score history', 'lifecycle-round-result', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.lastRound) frame.room.lastRound.result = { type: 'draw', reason: 'timeout' }
    }],
    ['score winner history', 'lifecycle-score-winner', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.lastRound?.result.type === 'winner') frame.room.lastRound.result.seat = 'two'
    }],
    ['forfeit reservation', 'lifecycle-forfeit', (frame: ReturnType<typeof roomState>) => {
      frame.room.seats[0]!.connection = { type: 'connected' }
      frame.room.settingsEditor = 'one'
    }],
    ['interrupted deadline', 'lifecycle-interrupted', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.phase.type === 'interrupted') frame.room.phase.endedAt += 1
    }],
    ['rematch consensus', 'lifecycle-rematch-eligible', (frame: ReturnType<typeof roomState>) => {
      frame.room.seats[2]!.rematchReady = true
    }],
    ['terminal disconnect TTL', 'lifecycle-rematch-eligible', (frame: ReturnType<typeof roomState>) => {
      if (frame.room.phase.type !== 'matchResult') throw new Error('expected match result')
      frame.room.seats[0]!.connection = {
        type: 'disconnected', expiresAt: frame.room.phase.endedAt + 601,
      }
      frame.room.seats[0]!.rematchReady = false
    }],
    ['disconnect TTL', 'lifecycle-disconnected', (frame: ReturnType<typeof roomState>) => {
      const connection = frame.room.seats[0]!.connection
      if (connection.type === 'disconnected') connection.expiresAt += 1
    }],
    ['disconnected readiness', 'lifecycle-disconnected', (frame: ReturnType<typeof roomState>) => {
      frame.room.seats[0]!.ready = true
    }],
    ['abandoned human reservation', 'lifecycle-abandoned', (frame: ReturnType<typeof roomState>) => {
      frame.room.seats[0]!.connection = { type: 'connected' }
      frame.room.settingsEditor = 'one'
    }],
  ] as const)('rejects invalid %s facts', (_label, caseId, mutate) => {
    const frame = roomState(caseId)
    mutate(frame)
    expect(validateGameplayV15ServerMessage(frame)).toMatchObject({ valid: false })
  })
})
