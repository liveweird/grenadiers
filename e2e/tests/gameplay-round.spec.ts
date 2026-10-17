import { expect, test } from '@playwright/test'
import type { GameServerMessage } from '../../web/src/game/schema'
import { gameFrames, openGame, roomFrame, sendGame } from './support/game'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>

function snapshot(frames: GameServerMessage[]): Snapshot | undefined {
  return frames.filter((frame) => frame.type === 'snapshot').at(-1)
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
    await sendGame(one, { type: 'ready', ...binding, ready: true })
    await sendGame(two, { type: 'ready', ...binding, ready: true })
    await expect.poll(async () => snapshot(await gameFrames(one))?.room.phase.type).toBe('running')
    await expect.poll(async () => snapshot(await gameFrames(two))?.room.phase.type).toBe('running')
    const firstRunning = snapshot(await gameFrames(one))!
    expect(firstRunning.room).toMatchObject({ rulesVersion: 2, mapId: 'generated_v1' })
    expect(firstRunning.state.pickups).toEqual([])
    const blocks = firstRunning.state.arena.blocks
    for (const block of blocks) {
      expect(blocks).toContainEqual({ column: 14 - block.column, row: 12 - block.row })
    }
    expect(blocks).not.toContainEqual({ column: 2, row: 1 })
    expect(blocks).not.toContainEqual({ column: 1, row: 2 })
    await expect.poll(async () => {
      await sendGame(one, { type: 'movement', ...binding, direction: 'east' })
      return snapshot(await gameFrames(one))?.state.actors[0].position.x ?? 0
    }, { intervals: [100] }).toBeGreaterThan(2350)
    await sendGame(one, { type: 'movement', ...binding, direction: null })
    await sendGame(one, { type: 'placeBomb', ...binding, actionId: 1 })
    await expect.poll(async () => snapshot(await gameFrames(one))?.state.bombs.length).toBe(1)
    const duplicate = await sendGame(one, { type: 'placeBomb', ...binding, actionId: 1 })
    await expect.poll(() => gameFrames(one)).toContainEqual({
      type: 'error', code: 'duplicateAction', sequence: duplicate,
    })
    await expect.poll(async () => snapshot(await gameFrames(one))?.room.phase.type).toBe('result')
    await expect.poll(async () => snapshot(await gameFrames(two))?.room.phase.type).toBe('result')
    const first = snapshot(await gameFrames(one))!
    expect(first.room.phase).toMatchObject({ type: 'result', result: { type: 'winner', seat: 'two' } })
    expect(first.state.actors[0].alive).toBe(false)
    await expect.poll(async () => snapshot(await gameFrames(one))!.room.revision).toBeGreaterThan(first.room.revision + 2)
    expect(snapshot(await gameFrames(one))!.state).toEqual(first.state)
    const other = (await gameFrames(two)).find((frame) =>
      frame.type === 'snapshot' && frame.room.revision === first.room.revision,
    )
    expect(other).toEqual(first)
    const events = (await gameFrames(one)).flatMap((frame) => frame.type === 'snapshot' ? frame.events : [])
    expect(events.some((event) => event.kind.type === 'bombPlaced')).toBe(true)
    expect(events.some((event) => event.kind.type === 'bombDetonated')).toBe(true)
    expect(events.some((event) => event.kind.type === 'roundEnded')).toBe(true)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
