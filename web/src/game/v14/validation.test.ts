import { describe, expect, it } from 'vitest'
import { snapshot } from './testSupport'
import type { ArenaView, BoardSize, Snapshot } from './schema'
import { validateGameplayV14ServerMessage } from './validation'

function fourPlayer(): Snapshot {
  const message = snapshot(2)
  message.room.rulesVersion = 6
  message.room.configuration = {
    controllers: ['human', 'cpu', 'human', 'cpu'], cpuDifficulty: 'standard',
    board: { width: 15, height: 13 },
  }
  message.room.seats = [
    message.room.seats[0]!,
    { ...message.room.seats[1]!, controller: { type: 'cpu', difficulty: 'standard' },
      connection: { type: 'serverControlled' }, ready: false },
    { seat: 'three', controller: { type: 'human' }, occupied: true, ready: false,
      rematchReady: false, connection: { type: 'connected' } },
    { seat: 'four', controller: { type: 'cpu', difficulty: 'standard' }, occupied: true,
      ready: false, rematchReady: false, connection: { type: 'serverControlled' } },
  ]
  message.room.scores = ['one', 'two', 'three', 'four'].map((seat) => ({
    seat: seat as 'one' | 'two' | 'three' | 'four', wins: 0,
  }))
  const one = message.state.actors[0]!
  const two = message.state.actors[1]!
  if (one.type !== 'visible' || two.type !== 'visible') throw new Error('fixture actors must be visible')
  message.state.actors.push(
    { ...one, id: 3, seat: 'three', position: { x: 512, y: 12_800 } },
    { ...two, id: 4, seat: 'four', position: { x: 14_848, y: 512 } },
  )
  return message
}

function setBoard(message: Snapshot, board: BoardSize): void {
  message.room.configuration.board = board
  message.state.arena = {
    ...board, terrain: Array.from({ length: board.width * board.height }, () => 'floor' as const), blocks: [],
  } as ArenaView
  const positions = [
    { x: 512, y: 512 }, { x: board.width * 1024 - 512, y: board.height * 1024 - 512 },
    { x: 512, y: board.height * 1024 - 512 }, { x: board.width * 1024 - 512, y: 512 },
  ]
  message.state.actors.forEach((actor, index) => {
    if (actor.type === 'visible') actor.position = positions[index]!
  })
}

function configured(count: 2 | 3 | 4, board: BoardSize): Snapshot {
  const message = fourPlayer()
  const controllers = count === 2 ? ['human', 'human'] as ['human', 'human']
    : count === 3 ? ['human', 'human', 'human'] as ['human', 'human', 'human']
      : ['human', 'human', 'human', 'human'] as ['human', 'human', 'human', 'human']
  message.room.configuration = {
    controllers, cpuDifficulty: null, board,
  }
  message.room.seats = message.room.seats.slice(0, count).map((seat) => ({
    ...seat, controller: { type: 'human' }, connection: { type: 'connected' },
  }))
  message.room.scores = message.room.scores.slice(0, count)
  message.state.actors = message.state.actors.slice(0, count)
  setBoard(message, board)
  return message
}

describe('gameplay v14 multiplayer semantics', () => {
  it('accepts an ordered mixed four-player roster', () => {
    const message = fourPlayer()
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([
    [2, { width: 11, height: 9 }], [2, { width: 13, height: 11 }], [2, { width: 15, height: 13 }],
    [3, { width: 11, height: 9 }], [3, { width: 13, height: 11 }], [3, { width: 15, height: 13 }],
    [4, { width: 11, height: 9 }], [4, { width: 13, height: 11 }], [4, { width: 15, height: 13 }],
  ] as const)('accepts rules 6 with %s seats on a $width × $height board', (count, board) => {
    const message = configured(count, board)
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('retains the legacy rules profile for a two-seat v14 room', () => {
    const message = snapshot(2)
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it.each([{ width: 11, height: 9 }, { width: 13, height: 11 }] as const)(
    'rejects a bodyless legacy room on a $width × $height board', (board) => {
      const message = snapshot(2)
      message.room.configuration.board = board
      expect(validateGameplayV14ServerMessage({ type: 'roomState', room: message.room })).toEqual({
        valid: false, source: 'semantics', error: 'unsupportedProfile',
      })
    },
  )

  it.each([3, 4])('rejects the legacy rules profile for a %s-seat room', (count) => {
    const message = fourPlayer()
    message.room.rulesVersion = 5
    message.room.configuration.controllers = count === 3
      ? ['human', 'cpu', 'human'] : ['human', 'cpu', 'human', 'cpu']
    message.room.seats = message.room.seats.slice(0, count)
    message.room.scores = message.room.scores.slice(0, count)
    message.state.actors = message.state.actors.slice(0, count)
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'unsupportedProfile',
    })
  })

  it('rejects mismatched configuration, roster, score, and bomb ownership facts', () => {
    for (const mutate of [
      (message: Snapshot) => { message.room.configuration.cpuDifficulty = 'easy' },
      (message: Snapshot) => { message.room.seats[2]!.seat = 'four' },
      (message: Snapshot) => { message.room.scores[2]!.seat = 'four' },
      (message: Snapshot) => { message.state.bombs = [{
        id: 20, cell: { column: 2, row: 1 }, owner: 99, range: 1, placedAt: 1,
        trigger: { type: 'timed', remainingTicks: 30, paused: false }, blast: 'standard',
      }] },
    ]) {
      const message = fourPlayer()
      mutate(message)
      expect(validateGameplayV14ServerMessage(message)).toMatchObject({ valid: false })
    }
  })

  it.each([
    ['arena block', (message: Snapshot) => { message.state.arena.blocks = [{ column: 12, row: 1 }] }],
    ['bomb', (message: Snapshot) => { message.state.bombs = [{
      id: 20, cell: { column: 12, row: 1 }, owner: 1, range: 1, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 30, paused: false }, blast: 'standard',
    }] }],
    ['flame', (message: Snapshot) => { message.state.flames = [{
      cell: { column: 12, row: 1 }, expiresAt: 2,
    }] }],
    ['pickup', (message: Snapshot) => { message.state.pickups = [{
      cell: { column: 12, row: 1 }, kind: 'bombCapacity',
    }] }],
    ['oil barrel', (message: Snapshot) => {
      message.room.settings.enabledItems = ['heart', 'oilBarrel']
      message.state.oilBarrels = [{ cell: { column: 12, row: 1 }, state: 'unlit' }]
    }],
    ['placed bomb event', (message: Snapshot) => { message.events = [{ tick: 1, index: 0, kind: {
      type: 'bombPlaced', bomb: { id: 20, cell: { column: 12, row: 1 }, owner: 1, range: 1,
        placedAt: 1, trigger: { type: 'timed', remainingTicks: 30, paused: false }, blast: 'standard' },
    } }] }],
    ['detonated bomb event', (message: Snapshot) => { message.events = [{ tick: 1, index: 0, kind: {
      type: 'bombDetonated', bomb: { id: 20, cell: { column: 12, row: 1 }, owner: 1, range: 1,
        placedAt: 1, trigger: { type: 'timed', remainingTicks: 30, paused: false }, blast: 'standard' },
    } }] }],
    ['pickup event', (message: Snapshot) => { message.events = [{ tick: 1, index: 0, kind: {
      type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 12, row: 1 }, kind: 'bombCapacity',
    } }] }],
  ] as const)('rejects a %s cell whose flattened offset aliases the next 11-wide row', (_name, mutate) => {
    const message = configured(2, { width: 11, height: 9 })
    mutate(message)
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidWorldState',
    })
  })

  it('accepts state and event cells on the inclusive lower-right board edge', () => {
    const message = configured(2, { width: 11, height: 9 })
    message.state.bombs = [{
      id: 20, cell: { column: 10, row: 8 }, owner: 1, range: 1, placedAt: 1,
      trigger: { type: 'timed', remainingTicks: 30, paused: false }, blast: 'standard',
    }]
    message.events = [{ tick: 1, index: 0, kind: {
      type: 'pickupCollected', id: 1, seat: 'one', cell: { column: 10, row: 8 }, kind: 'bombCapacity',
    } }]
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('accepts inclusive hitbox edges and rejects each board-relative overflow direction', () => {
    const atEdges = configured(2, { width: 11, height: 9 })
    const actor = atEdges.state.actors[0]!
    if (actor.type !== 'visible') throw new Error('fixture actor must be visible')
    actor.position = { x: 320, y: 320 }
    expect(validateGameplayV14ServerMessage(atEdges)).toEqual({ valid: true, value: atEdges })
    actor.position = { x: 11 * 1024 - 320, y: 9 * 1024 - 320 }
    expect(validateGameplayV14ServerMessage(atEdges)).toEqual({ valid: true, value: atEdges })

    for (const position of [
      { x: 319, y: 512 }, { x: 11 * 1024 - 319, y: 512 },
      { x: 512, y: 319 }, { x: 512, y: 9 * 1024 - 319 },
    ]) {
      const outside = configured(2, { width: 11, height: 9 })
      const candidate = outside.state.actors[0]!
      if (candidate.type !== 'visible') throw new Error('fixture actor must be visible')
      candidate.position = position
      expect(validateGameplayV14ServerMessage(outside)).toMatchObject({
        valid: false, source: 'semantics', error: 'invalidWorldState',
      })
    }
  })

  it('retains creator-only settings authority after other humans connect', () => {
    const message = fourPlayer()
    message.room.settingsEditor = 'three'
    expect(validateGameplayV14ServerMessage(message)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidControllerMetadata',
    })
    message.room.settingsEditor = null
    message.room.seats[0]!.connection = { type: 'expired' }
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('accepts an authoritative freeze when the Stopwatch holder is hidden by recipient privacy', () => {
    const message = fourPlayer()
    message.room.settings.enabledItems = ['heart', 'stopwatch', 'invisibilityCloak']
    const hidden = message.state.actors[1]!
    message.state.actors[1] = { type: 'hidden', id: hidden.id, seat: hidden.seat, alive: true }
    for (const actor of message.state.actors) {
      if (actor.type === 'visible') actor.voluntaryActionsFrozen = true
    }
    expect(validateGameplayV14ServerMessage(message)).toEqual({ valid: true, value: message })
  })

  it('still rejects a known missing freeze from a visible Stopwatch holder', () => {
    const message = fourPlayer()
    message.room.settings.enabledItems = ['heart', 'stopwatch']
    const holder = message.state.actors[1]!
    if (holder.type !== 'visible') throw new Error('fixture holder must be visible')
    holder.effects = [{ kind: 'stopwatch', remainingTicks: 180 }]
    for (const actor of message.state.actors.slice(2)) {
      if (actor.type === 'visible') actor.voluntaryActionsFrozen = true
    }
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })
  })

  it('rejects a forged freeze with only a privacy-hidden Cloak opponent', () => {
    const message = fourPlayer()
    message.room.settings.enabledItems = ['heart', 'invisibilityCloak']
    const hidden = message.state.actors[1]!
    message.state.actors[1] = { type: 'hidden', id: hidden.id, seat: hidden.seat, alive: true }
    const victim = message.state.actors[0]!
    if (victim.type !== 'visible') throw new Error('fixture victim must be visible')
    victim.voluntaryActionsFrozen = true
    expect(validateGameplayV14ServerMessage(message)).toEqual({
      valid: false, source: 'semantics', error: 'invalidFrozenState',
    })
  })
})
