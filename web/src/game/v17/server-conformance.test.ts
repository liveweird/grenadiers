import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v17-server-conformance.json'
import type { GameServerMessage, Snapshot, VisibleActorView } from './schema'
import { validateGameplayV17ServerMessage } from './validation'

type Case = { caseId: string; scenario: string; recipient: string | null; frame: string }
const cases = corpus.cases as Case[]
const frames = new Map(cases.map(({ caseId, frame }) => [caseId, JSON.parse(frame) as GameServerMessage]))

function snapshot(caseId: string): Snapshot {
  const frame = structuredClone(frames.get(caseId))
  if (frame?.type !== 'snapshot') throw new Error(`missing snapshot fixture ${caseId}`)
  return frame
}

function visible(frame: Snapshot): VisibleActorView[] {
  return frame.state.actors.filter((actor): actor is VisibleActorView => actor.type === 'visible')
}

describe('Rust-produced gameplay v17 server conformance', () => {
  it('accepts every authoritative Rust projection frame', () => {
    expect(cases).toHaveLength(30)
    for (const entry of cases) {
      const frame = frames.get(entry.caseId)
      expect(validateGameplayV17ServerMessage(frame), entry.caseId).toEqual({ valid: true, value: frame })
    }
  })

  it('covers protocol17/rules9 lifecycle, boards, privacy, and CPU21 metadata', () => {
    const messages = [...frames.values()]
    expect(messages.some((frame) => frame.type === 'admitted')).toBe(true)
    expect(messages.some((frame) => frame.type === 'roomState')).toBe(true)
    const scenarios = new Set(cases.map(({ scenario }) => scenario))
    for (const scenario of [
      'waitingAdmission', 'initialCountdown', 'roundResult', 'interRoundCountdown',
      'nextRunning', 'scoreWinner', 'forfeit', 'interrupted', 'rematchEligible',
      'disconnected', 'abandoned', 'resultAdmission', 'continuedForfeitAdmission',
    ]) expect(scenarios.has(scenario), scenario).toBe(true)
    const rooms = messages.flatMap((frame) => 'room' in frame ? [frame.room] : [])
    expect(rooms.every(({ rulesVersion }) => rulesVersion === 9)).toBe(true)
    expect(new Set(rooms.map(({ configuration }) =>
      `${configuration.board.width}x${configuration.board.height}`,
    ))).toEqual(new Set(['11x9', '13x11', '15x13']))
    expect(rooms.flatMap(({ seats }) => seats).filter(({ controller }) => controller.type === 'cpu')
      .every(({ connection }) => connection.type === 'serverControlled')).toBe(true)
  })

  it('proves occupied null timers and independent empty-lair cooldowns', () => {
    expect(snapshot('populated11x9Lairs-one').state.lairs).toEqual([
      { id: 0, cell: { column: 4, row: 4 }, spawnRemainingTicks: null, livingMobs: 1 },
      { id: 1, cell: { column: 6, row: 4 }, spawnRemainingTicks: null, livingMobs: 1 },
    ])
    expect(snapshot('oneLairCoolingOtherOccupied-one').state.lairs).toEqual([
      { id: 0, cell: { column: 4, row: 4 }, spawnRemainingTicks: 300, livingMobs: 0 },
      { id: 1, cell: { column: 6, row: 4 }, spawnRemainingTicks: null, livingMobs: 1 },
    ])
    const one = snapshot('independentDeathCooldowns-one').state.lairs
    const two = snapshot('independentDeathCooldowns-two').state.lairs
    expect(one.map(({ spawnRemainingTicks }) => spawnRemainingTicks)).toEqual([290, 300])
    expect(two).toEqual(one)
  })

  it('proves eligible zero retries and first-clear spawning without a burst', () => {
    const blocked = snapshot('blockedEligibleLairs-one')
    expect(blocked.state.lairs.map(({ spawnRemainingTicks }) => spawnRemainingTicks)).toEqual([0, 0])
    expect(blocked.state.mobs).toEqual([])
    const cleared = snapshot('spawnOnFirstClearTick-one')
    expect(cleared.state.lairs.map(({ spawnRemainingTicks }) => spawnRemainingTicks)).toEqual([null, null])
    expect(cleared.state.mobs).toHaveLength(2)
    expect(new Set(cleared.state.mobs.map(({ lairId }) => lairId))).toEqual(new Set([0, 1]))
  })

  it('proves Stopwatch freeze and recipient-safe cloak projection', () => {
    expect(snapshot('frozen11x9Lairs-one').state.mobs.every(({ frozen }) => frozen)).toBe(true)
    const one = snapshot('single15x13Cpu21Cloak-one')
    const two = snapshot('single15x13Cpu21Cloak-two')
    expect(one.state.actors.find(({ seat }) => seat === 'two')).toMatchObject({ type: 'hidden', alive: true })
    expect(two.state.actors.find(({ seat }) => seat === 'two')).toMatchObject({ type: 'visible' })
  })

  it('proves dense mines are extra to a full ordinary slot and privacy-safe per recipient', () => {
    const visible = snapshot('denseMinesMixedVisibility-two')
    expect(visible.state.bombs).toHaveLength(34)
    expect(visible.state.bombs.filter(({ trigger }) => trigger.type === 'contact')).toHaveLength(33)
    expect(visible.state.bombs.filter(({ trigger }) => trigger.type !== 'contact')).toHaveLength(1)
    expect(visible.state.actors.find(({ seat }) => seat === 'two')).toMatchObject({
      type: 'visible', bombCapacity: 1, liveBombCount: 1,
    })
    expect(snapshot('denseMinesMixedVisibility-one').state.bombs).toEqual([])
    expect(snapshot('denseMinesCloakedOwner-one').state.bombs).toEqual([])
  })

  it.each([
    ['occupied countdown', (frame: Snapshot) => { frame.state.lairs[0]!.spawnRemainingTicks = 1 }],
    ['empty null timer', (frame: Snapshot) => {
      frame.state.mobs = []; frame.state.lairs[0]!.livingMobs = 0
    }],
    ['lair count mismatch', (frame: Snapshot) => { frame.state.lairs[0]!.livingMobs = 0 }],
    ['same-cell players', (frame: Snapshot) => { visible(frame)[1]!.position = visible(frame)[0]!.position }],
    ['positive-area overlap', (frame: Snapshot) => {
      visible(frame)[1]!.position = { x: visible(frame)[0]!.position.x + 639,
        y: visible(frame)[0]!.position.y + 639 }
    }],
  ] as const)('rejects authoritative-state mutation: %s', (_label, mutate) => {
    const frame = snapshot('populated11x9Lairs-one')
    mutate(frame)
    expect(validateGameplayV17ServerMessage(frame)).toMatchObject({ valid: false })
  })

  it.each([
    ['ready vacant seat', (frame: Snapshot) => {
      Object.assign(frame.room.seats[0]!, {
        occupied: false, ready: true, connection: { type: 'vacant' },
      })
    }, 'invalidControllerMetadata'],
    ['human seat labelled CPU', (frame: Snapshot) => {
      frame.room.seats[0]!.controller = { type: 'cpu', difficulty: 'standard' }
    }, 'invalidControllerMetadata'],
    ['unknown bomb owner', (frame: Snapshot) => {
      frame.state.bombs.push({
        id: 99, owner: 999, cell: { column: 1, row: 1 }, range: 1,
        placedAt: frame.state.physicalTick, blast: 'standard',
        trigger: { type: 'timed', remainingTicks: 1, paused: false },
      })
    }, 'invalidWorldState'],
    ['snapshot clock drift', (frame: Snapshot) => { frame.state.physicalTick += 1 }, 'invalidStateBinding'],
  ] as const)('rejects inherited rules9 guard: %s', (_label, mutate, error) => {
    const frame = snapshot('populated11x9Lairs-one')
    mutate(frame)
    expect(validateGameplayV17ServerMessage(frame)).toEqual({ valid: false, source: 'semantics', error })
  })

  it('rejects disabled lair evidence and disabled storm timers', () => {
    const lair = snapshot('disabled11x9Cpu21-one')
    lair.state.lairs.push({
      id: 0, cell: { column: 4, row: 4 }, spawnRemainingTicks: 0, livingMobs: 0,
    })
    expect(validateGameplayV17ServerMessage(lair)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidLairState',
    })

    const storm = snapshot('disabled11x9Cpu21-one')
    visible(storm)[0]!.stormSlowRemainingTicks = 1
    expect(validateGameplayV17ServerMessage(storm)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidStormEnvironment',
    })
  })

  it('rejects false event identity and impossible oil pickup events', () => {
    const identity = snapshot('populated11x9Lairs-one')
    identity.events.push({
      tick: identity.state.physicalTick, index: 0,
      kind: { type: 'actorEliminated', id: identity.state.actors[0]!.id, seat: 'two' },
    })
    expect(validateGameplayV17ServerMessage(identity)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidWorldState',
    })

    const oil = snapshot('populated11x9Lairs-one')
    oil.events.push({ tick: oil.state.physicalTick, index: 0, kind: {
      type: 'pickupCollected', id: oil.state.actors[0]!.id, seat: 'one',
      cell: { column: 1, row: 1 }, kind: 'oilBarrel',
    } })
    expect(validateGameplayV17ServerMessage(oil)).toMatchObject({
      valid: false, source: 'semantics', error: 'invalidOilBarrels',
    })
  })
})
