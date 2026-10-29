import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v16-server-conformance.json'
import type { GameServerMessage, Snapshot } from './schema'
import { validateGameplayV16ServerMessage } from './validation'

type Case = { caseId: string; scenario: string; recipient: string | null; frame: string }

const cases = corpus.cases as Case[]
const frames = new Map(cases.map((entry) => [entry.caseId, JSON.parse(entry.frame) as GameServerMessage]))

function snapshot(caseId: string): Snapshot {
  const frame = structuredClone(frames.get(caseId))
  if (frame?.type !== 'snapshot') throw new Error(`missing snapshot fixture ${caseId}`)
  return frame
}

describe('Rust-produced gameplay v16 server conformance', () => {
  it('accepts every freshly reproducible checked Rust frame', () => {
    for (const entry of cases) {
      const frame = frames.get(entry.caseId)
      expect(validateGameplayV16ServerMessage(frame), entry.caseId).toEqual({
        valid: true, value: frame,
      })
    }
  })

  it('covers admissions, room state, every board/count, both environments, and CPU18 metadata', () => {
    const messages = [...frames.values()]
    expect(messages.some((frame) => frame.type === 'admitted')).toBe(true)
    expect(messages.some((frame) => frame.type === 'roomState')).toBe(true)
    const rooms = messages.flatMap((frame) => 'room' in frame ? [frame.room] : [])
    expect(new Set(rooms.map(({ configuration }) =>
      `${configuration.board.width}x${configuration.board.height}:${configuration.controllers.length}`,
    ))).toEqual(new Set(['11x9:2', '11x9:3', '13x11:3', '15x13:4']))
    expect(new Set(rooms.map(({ settings }) => settings.environment.stormCloud))).toEqual(
      new Set([false, true]),
    )
    expect(rooms.every(({ rulesVersion }) => rulesVersion === 8)).toBe(true)
    expect(rooms.flatMap(({ seats }) => seats)
      .filter(({ controller }) => controller.type === 'cpu')
      .every(({ connection }) => connection.type === 'serverControlled')).toBe(true)
  })

  it('proves real storm contact and cloak timer privacy for selected recipients', () => {
    const disabled = snapshot('disabled11x9Cpu18-one')
    expect(disabled.state.stormCloud).toBeNull()
    expect(disabled.state.actors.every((actor) =>
      actor.type === 'hidden' || actor.stormSlowRemainingTicks === 0,
    )).toBe(true)

    const contacted = snapshot('single13x11Cpu18Timer-one')
    const own = contacted.state.actors.find(({ seat }) => seat === 'one')
    expect(own).toMatchObject({ type: 'visible', stormSlowRemainingTicks: 180 })

    const opponent = snapshot('single15x13Cpu18Cloak-one')
    expect(opponent.state.actors.find(({ seat }) => seat === 'two')).toEqual({
      type: 'hidden', id: 11, seat: 'two', alive: true,
    })
    const cloakedOwn = snapshot('single15x13Cpu18Cloak-two')
    expect(cloakedOwn.state.actors.find(({ seat }) => seat === 'two')).toMatchObject({
      type: 'visible', stormSlowRemainingTicks: 180,
    })
  })

  it('proves Rust-produced canonical lairs, spawned mobs, timers, IDs, and positions', () => {
    const disabled = snapshot('disabled11x9Cpu18-one')
    expect(disabled.state.lairs).toEqual([])
    expect(disabled.state.mobs).toEqual([])

    const populated = snapshot('populated11x9Lairs-one')
    expect(populated.state.lairs).toEqual([
      { id: 0, cell: { column: 4, row: 4 }, spawnRemainingTicks: 300, livingMobs: 1 },
      { id: 1, cell: { column: 6, row: 4 }, spawnRemainingTicks: 300, livingMobs: 1 },
    ])
    expect(populated.state.mobs).toEqual([
      { id: 0, lairId: 0, position: { x: 3584, y: 4608 }, heading: 'south', frozen: false },
      { id: 1, lairId: 1, position: { x: 7680, y: 4608 }, heading: 'south', frozen: false },
    ])
    expect(snapshot('frozen11x9Lairs-one').state.mobs.every(({ frozen }) => frozen)).toBe(true)
  })

  it.each([
    ['configuration', (frame: Snapshot) => { frame.room.configuration.environment.lairs.enabled = false }],
    ['layout', (frame: Snapshot) => { frame.state.lairs[0]!.cell.column += 1 }],
    ['timer', (frame: Snapshot) => {
      for (const lair of frame.state.lairs) lair.spawnRemainingTicks = 301
    }],
    ['unsynchronized timer', (frame: Snapshot) => { frame.state.lairs[0]!.spawnRemainingTicks = 299 }],
    ['count', (frame: Snapshot) => { frame.state.lairs[0]!.livingMobs = 2 }],
    ['lair ID', (frame: Snapshot) => { frame.state.lairs[0]!.id = 1 }],
    ['mob ID collision', (frame: Snapshot) => { frame.state.mobs[0]!.id = frame.state.actors[0]!.id }],
    ['mob lair', (frame: Snapshot) => { frame.state.mobs[0]!.lairId = 1 }],
    ['mob position', (frame: Snapshot) => { frame.state.mobs[0]!.position.x = 65_535 }],
    ['global frozen state', (frame: Snapshot) => { frame.state.mobs[0]!.frozen = true }],
  ] as const)('rejects lair mutation: %s', (_label, mutate) => {
    const frame = snapshot('populated11x9Lairs-one')
    mutate(frame)
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it('does not infer a hidden actor effect deadline when all mobs are globally frozen', () => {
    const frame = snapshot('populated11x9Lairs-one')
    const actor = frame.state.actors[1]!
    frame.state.actors[1] = {
      type: 'hidden', id: actor.id, seat: actor.seat, alive: actor.alive,
    }
    frame.room.settings.enabledItems.push('invisibilityCloak')
    for (const mob of frame.state.mobs) mob.frozen = true
    expect(validateGameplayV16ServerMessage(frame)).toEqual({ valid: true, value: frame })
  })

  it('rejects false derived CPU readiness before the first start', () => {
    const frame = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    if (frame?.type !== 'admitted') throw new Error('missing waiting admission fixture')
    frame.room.seats[1]!.ready = false
    expect(validateGameplayV16ServerMessage(frame)).toEqual({
      valid: false, source: 'semantics', error: 'invalidControllerMetadata',
    })
  })

  it('rejects scores and history in the actual waiting lifecycle fixture', () => {
    const score = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    if (score?.type !== 'admitted') throw new Error('missing waiting admission fixture')
    score.room.scores[0]!.wins = 1
    expect(validateGameplayV16ServerMessage(score)).toMatchObject({ valid: false })

    const history = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    if (history?.type !== 'admitted') throw new Error('missing waiting admission fixture')
    history.room.lastRound = {
      roundId: 1, roundNumber: 1, endedAt: 0, result: { type: 'winner', seat: 'one' },
    }
    expect(validateGameplayV16ServerMessage(history)).toMatchObject({ valid: false })
  })

  it('accepts only a future bounded initial countdown with readiness consensus', () => {
    const frame = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    if (frame?.type !== 'admitted') throw new Error('missing waiting admission fixture')
    frame.room.seats[0]!.ready = true
    frame.room.phase = { type: 'countdown', kind: 'initial', endsAt: 180 }
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: true })
    frame.room.phase.endsAt = 0
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it.each([
    ['waiting round one', (frame: GameServerMessage) => {
      if (!('room' in frame)) return
      frame.room.phase = { type: 'waiting' }
    }],
    ['score above finish', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.scores[0]!.wins = 4
    }],
    ['empty item availability', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.availableItems = []
    }],
    ['connected unoccupied seat', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.seats[0]!.occupied = false
    }],
    ['human ready while running', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.seats[0]!.ready = true
    }],
    ['CPU ready while running', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.seats[1]!.ready = true
    }],
    ['human rematch vote while running', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.seats[0]!.rematchReady = true
    }],
    ['disconnect beyond TTL', (frame: GameServerMessage) => {
      if ('room' in frame) frame.room.seats[0]!.connection = {
        type: 'disconnected', expiresAt: frame.room.tick + 601,
      }
    }],
  ] as const)('rejects lifecycle mutation: %s', (_label, mutate) => {
    const frame = structuredClone(frames.get('disabled11x9Cpu18-one'))
    if (!frame) throw new Error('missing running fixture')
    mutate(frame)
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it('rejects terminal winners outside the roster and an invalid rematch deadline', () => {
    const frame = snapshot('disabled11x9Cpu18-one')
    const room = frame.room
    room.tick = 181
    room.roundId = 3
    room.roundNumber = 3
    room.scores[0]!.wins = 3
    room.lastRound = {
      roundId: 3, roundNumber: 3, endedAt: 181, result: { type: 'winner', seat: 'one' },
    }
    room.phase = {
      type: 'matchResult', endedAt: 181,
      result: { type: 'winner', seat: 'one', reason: 'roundWins' }, rematchAt: 361,
    }
    frame.state.physicalTick = 181
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: true })

    const wrongPhysicalTick = structuredClone(frame)
    wrongPhysicalTick.state.physicalTick = 180
    expect(validateGameplayV16ServerMessage(wrongPhysicalTick)).toEqual({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const badWinner = structuredClone(frame)
    if (badWinner.room.phase.type !== 'matchResult') throw new Error('expected terminal phase')
    badWinner.room.phase.result = { type: 'winner', seat: 'four', reason: 'roundWins' }
    expect(validateGameplayV16ServerMessage(badWinner)).toMatchObject({ valid: false })

    if (frame.room.phase.type !== 'matchResult') throw new Error('expected terminal phase')
    frame.room.phase.rematchAt += 1
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })
})
describe('gameplay v16 message-state binding', () => {
  it('binds snapshot and admission bodies to phases and retained-state boundaries', () => {
    const waiting = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    const running = snapshot('disabled11x9Cpu18-one')
    if (waiting?.type !== 'admitted') throw new Error('missing waiting admission fixture')

    const waitingSnapshot: Snapshot = {
      type: 'snapshot', room: waiting.room, state: running.state, events: [],
    }
    expect(validateGameplayV16ServerMessage(waitingSnapshot)).toEqual({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const waitingWithAction = structuredClone(waiting)
    waitingWithAction.lastActionId = 1
    expect(validateGameplayV16ServerMessage(waitingWithAction)).toEqual({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const waitingWithBody = structuredClone(waiting)
    waitingWithBody.state = running.state
    expect(validateGameplayV16ServerMessage(waitingWithBody)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const runningWithoutBody: GameServerMessage = {
      type: 'admitted', sequence: 1, room: running.room, youSeat: 'one',
      resumeToken: 'c'.repeat(64), lastActionId: null, state: null,
    }
    expect(validateGameplayV16ServerMessage(runningWithoutBody)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const wrongRunningTick = structuredClone(running)
    wrongRunningTick.state.physicalTick += 1
    expect(validateGameplayV16ServerMessage(wrongRunningTick)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const runningWithAction: GameServerMessage = {
      type: 'admitted', sequence: 1, room: running.room, youSeat: 'one',
      resumeToken: 'c'.repeat(64), lastActionId: 1, state: running.state,
    }
    expect(validateGameplayV16ServerMessage(runningWithAction)).toMatchObject({ valid: true })

    const interruptedSnapshot = structuredClone(waitingSnapshot)
    interruptedSnapshot.room.seats[1]!.ready = false
    interruptedSnapshot.room.phase = { type: 'interrupted', endedAt: 0, reason: 'internalError' }
    expect(validateGameplayV16ServerMessage(interruptedSnapshot)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })
  })

  it('allows only explicit null-admission phases and binds lastActionId to the current epoch', () => {
    const waiting = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    if (waiting?.type !== 'admitted') throw new Error('missing waiting admission fixture')
    waiting.room.tick = 180
    waiting.room.roundNumber = 1
    waiting.room.seats[1]!.ready = false
    waiting.room.phase = {
      type: 'matchResult', endedAt: 180,
      result: { type: 'interrupted', reason: 'internalError' }, rematchAt: 360,
    }
    expect(validateGameplayV16ServerMessage(waiting)).toMatchObject({ valid: true })
    waiting.room.phase.result = { type: 'interrupted', reason: 'serverStopped' }
    expect(validateGameplayV16ServerMessage(waiting)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })

    const retained = snapshot('disabled11x9Cpu18-one')
    retained.room.tick = 181
    retained.room.roundId = 2
    retained.room.scores[0]!.wins = 1
    retained.room.lastRound = {
      roundId: 1, roundNumber: 1, endedAt: 180, result: { type: 'winner', seat: 'one' },
    }
    retained.room.phase = { type: 'result', holdUntil: 360 }
    expect(validateGameplayV16ServerMessage(retained)).toMatchObject({ valid: true })
    const wrongRetainedTick = structuredClone(retained)
    wrongRetainedTick.state.physicalTick = retained.room.tick
    expect(validateGameplayV16ServerMessage(wrongRetainedTick)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })
    const admission: GameServerMessage = {
      type: 'admitted', sequence: 1, room: retained.room, youSeat: 'one',
      resumeToken: 'c'.repeat(64), lastActionId: null, state: retained.state,
    }
    expect(validateGameplayV16ServerMessage(admission)).toMatchObject({ valid: true })
    admission.lastActionId = 1
    expect(validateGameplayV16ServerMessage(admission)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStateBinding',
    })
  })
})

describe('gameplay v16 environment mutation rejection', () => {
  it.each([
    ['environment mismatch', (frame: Snapshot) => {
      frame.room.configuration.environment.stormCloud = false
    }],
    ['cloud outside interior bounds', (frame: Snapshot) => {
      frame.state.stormCloud!.position.x = 512
    }],
    ['cloud leg and heading inconsistency', (frame: Snapshot) => {
      frame.state.stormCloud!.legRemainingTicks = 126
    }],
    ['timer range overflow', (frame: Snapshot) => {
      const actor = frame.state.actors.find((candidate) => candidate.type === 'visible')
      if (actor?.type === 'visible') actor.stormSlowRemainingTicks = 181
    }],
    ['hidden timer leak', (frame: Snapshot) => {
      const actor = frame.state.actors.find(({ seat }) => seat === 'two')
      Object.assign(actor!, { stormSlowRemainingTicks: 12 })
    }],
  ] as const)('rejects the negative mutation: %s', (_label, mutate) => {
    const frame = snapshot('single15x13Cpu18Cloak-one')
    mutate(frame)
    expect(validateGameplayV16ServerMessage(frame)).toMatchObject({ valid: false })
  })
})
