import { describe, expect, it } from 'vitest'
import corpus from '../../../../api/fixtures/gameplay-v15-server-conformance.json'
import type { GameServerMessage, Snapshot } from './schema'
import { validateGameplayV15ServerMessage } from './validation'

type Case = { caseId: string; scenario: string; recipient: string | null; frame: string }

const cases = corpus.cases as Case[]
const frames = new Map(cases.map((entry) => [entry.caseId, JSON.parse(entry.frame) as GameServerMessage]))

function snapshot(caseId: string): Snapshot {
  const frame = structuredClone(frames.get(caseId))
  if (frame?.type !== 'snapshot') throw new Error(`missing snapshot fixture ${caseId}`)
  return frame
}

describe('Rust-produced gameplay v15 server conformance', () => {
  it('accepts every freshly reproducible checked Rust frame', () => {
    for (const entry of cases) {
      const frame = frames.get(entry.caseId)
      expect(validateGameplayV15ServerMessage(frame), entry.caseId).toEqual({
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
    ))).toEqual(new Set(['11x9:2', '13x11:3', '15x13:4']))
    expect(new Set(rooms.map(({ settings }) => settings.environment.stormCloud))).toEqual(
      new Set([false, true]),
    )
    expect(rooms.every(({ rulesVersion }) => rulesVersion === 7)).toBe(true)
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

  it('rejects false derived CPU readiness before the first start', () => {
    const frame = structuredClone(frames.get('disabled-11x9-cpu18-admission'))
    if (frame?.type !== 'admitted') throw new Error('missing waiting admission fixture')
    frame.room.seats[1]!.ready = false
    expect(validateGameplayV15ServerMessage(frame)).toEqual({
      valid: false, source: 'semantics', error: 'invalidControllerMetadata',
    })
  })

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
    expect(validateGameplayV15ServerMessage(frame)).toMatchObject({ valid: false })
  })
})
