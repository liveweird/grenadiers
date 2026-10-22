import { expect, test } from '@playwright/test'
import type { GameServerMessage } from '../../web/src/game/v10/schema'
import { gameFrames, openGame, roomFrame, sendGame } from './support/game'
import { visibleActor } from './support/duel'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>

function snapshot(frames: GameServerMessage[]): Snapshot | undefined {
  return frames.filter((frame) => frame.type === 'snapshot').at(-1)
}

function firstResultForRound(frames: GameServerMessage[], roundId: number): Snapshot | undefined {
  return frames.find((frame): frame is Snapshot => frame.type === 'snapshot'
    && frame.room.roundId === roundId
    && frame.room.phase.type === 'result')
}

function laterResultForRound(
  frames: GameServerMessage[],
  roundId: number,
  afterRevision: number,
): Snapshot | undefined {
  return frames.findLast((frame): frame is Snapshot => frame.type === 'snapshot'
    && frame.room.roundId === roundId
    && frame.room.phase.type === 'result'
    && frame.room.revision > afterRevision)
}

function resultForRoundAtRevision(
  frames: GameServerMessage[],
  roundId: number,
  revision: number,
): Snapshot | undefined {
  return frames.findLast((frame): frame is Snapshot => frame.type === 'snapshot'
    && frame.room.roundId === roundId
    && frame.room.phase.type === 'result'
    && frame.room.revision === revision)
}

test('two native gameplay sockets finish one authoritative generated classic round', async ({ browser }) => {
  const contexts = await Promise.all([browser.newContext(), browser.newContext()])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two].map(openGame))
    await sendGame(one, { type: 'createRoom' })
    const created = await roomFrame(one)
    await sendGame(two, { type: 'joinRoom', roomCode: created.room.roomCode })
    const joined = await roomFrame(two)
    const binding = { roomId: created.room.roomId, roundId: joined.room.roundId }
    expect(joined.room.settings.enabledItems).toEqual(['heart'])
    await sendGame(one, {
      type: 'updateSettings', ...binding, settingsRevision: joined.room.settingsRevision,
      settings: { ...joined.room.settings, enabledItems: [] },
    })
    const configured = await roomFrame(one, joined.room.revision)
    expect(configured.room.settings.enabledItems).toEqual([])
    await sendGame(one, { type: 'ready', ...binding, ready: true, settingsRevision: configured.room.settingsRevision })
    await sendGame(two, { type: 'ready', ...binding, ready: true, settingsRevision: configured.room.settingsRevision })
    await expect.poll(async () => snapshot(await gameFrames(one))?.room.phase.type).toBe('running')
    await expect.poll(async () => snapshot(await gameFrames(two))?.room.phase.type).toBe('running')
    const firstRunning = snapshot(await gameFrames(one))!
    expect(firstRunning.room).toMatchObject({ rulesVersion: 5, mapId: 'generated_v2' })
    expect(firstRunning.room.availableItems).toEqual([
      'kick', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse',
      'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
    ])
    expect(firstRunning.room.settings.enabledItems).toEqual([])
    for (const actor of firstRunning.state.actors) {
      expect(actor).toMatchObject({
        type: 'visible', inventory: { equipment: null, weapon: null },
        effects: [], damageGraceRemainingTicks: 0,
      })
    }
    expect(firstRunning.state.pickups).toEqual([])
    const blocks = firstRunning.state.arena.blocks
    for (const block of blocks) {
      expect(blocks).toContainEqual({ column: 14 - block.column, row: 12 - block.row })
    }
    expect(blocks).not.toContainEqual({ column: 2, row: 1 })
    expect(blocks).not.toContainEqual({ column: 1, row: 2 })
    await expect.poll(async () => {
      await sendGame(one, { type: 'movement', ...binding, direction: 'east' })
      return visibleActor(snapshot(await gameFrames(one)), 'one')?.position.x ?? 0
    }, { intervals: [100] }).toBeGreaterThan(2350)
    await sendGame(one, { type: 'movement', ...binding, direction: null })
    const equipment = await sendGame(one, { type: 'manualAction', action: 'equipment', ...binding, actionId: 1 })
    await expect.poll(() => gameFrames(one)).toContainEqual({
      type: 'error', code: 'invalidMessage', sequence: equipment,
    })
    await sendGame(one, { type: 'manualAction', action: 'bomb', ...binding, actionId: 1 })
    await expect.poll(async () => snapshot(await gameFrames(one))?.state.bombs.length).toBe(1)
    const duplicate = await sendGame(one, { type: 'manualAction', action: 'bomb', ...binding, actionId: 1 })
    await expect.poll(() => gameFrames(one)).toContainEqual({
      type: 'error', code: 'duplicateAction', sequence: duplicate,
    })
    let firstResult: Snapshot | undefined
    await expect.poll(async () => {
      firstResult = firstResultForRound(await gameFrames(one), binding.roundId)
      return Boolean(firstResult)
    }).toBe(true)
    if (!firstResult) throw new Error('No result snapshot was received for the active round')
    const first = firstResult
    expect(first.room.lastRound?.result).toEqual({ type: 'winner', seat: 'two' })
    expect(first.room.scores).toEqual({ one: 0, two: 1 })
    expect(first.state.actors[0].alive).toBe(false)
    let laterResult: Snapshot | undefined
    await expect.poll(async () => {
      laterResult = laterResultForRound(
        await gameFrames(one),
        first.room.roundId,
        first.room.revision + 2,
      )
      return Boolean(laterResult)
    }).toBe(true)
    expect(laterResult?.state).toEqual(first.state)
    let other: Snapshot | undefined
    await expect.poll(async () => {
      other = resultForRoundAtRevision(
        await gameFrames(two),
        first.room.roundId,
        first.room.revision,
      )
      return Boolean(other)
    }).toBe(true)
    expect(other).toEqual(first)
    const events = (await gameFrames(one)).flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    expect(events.some((event) => event.kind.type === 'bombPlaced')).toBe(true)
    expect(events.some((event) => event.kind.type === 'bombDetonated')).toBe(true)
    expect(events.some((event) => event.kind.type === 'roundEnded')).toBe(true)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
