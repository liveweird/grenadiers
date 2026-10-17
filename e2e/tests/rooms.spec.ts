import { expect, test } from '@playwright/test'
import { gameFrames, openGame, roomFrame, sendGame } from './support/game'

test('rooms bind two independent players and preserve authentication after leaving', async ({ browser }) => {
  const contexts = await Promise.all(Array.from({ length: 3 }, () => browser.newContext()))
  try {
    const [one, two, third] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two, third].map(openGame))
    await sendGame(one, { type: 'createRoom' })
    const created = await roomFrame(one)
    expect(created.youSeat).toBe('one')
    expect(created.room.roomId).toMatch(/^[0-9a-f]{32}$/)
    expect(created.room.roomCode).toMatch(/^[A-HJ-NP-Z2-9]{8}$/)
    expect(created.room.phase).toEqual({ type: 'waiting' })
    expect(created.room.seats.map((seat) => seat.occupied)).toEqual([true, false])

    await sendGame(two, { type: 'joinRoom', roomCode: created.room.roomCode })
    const joined = await roomFrame(two)
    const shared = await roomFrame(one, created.room.revision)
    expect(joined.youSeat).toBe('two')
    expect(shared.youSeat).toBe('one')
    expect(joined.room).toEqual(shared.room)
    expect(joined.room.seats.map((seat) => seat.occupied)).toEqual([true, true])

    const deniedSequence = await sendGame(third, { type: 'joinRoom', roomCode: created.room.roomCode })
    await expect.poll(() => gameFrames(third)).toContainEqual({
      type: 'error', code: 'roomFull', sequence: deniedSequence,
    })
    const leftSequence = await sendGame(two, {
      type: 'leaveRoom', roomId: joined.room.roomId, roundId: joined.room.roundId,
    })
    await expect.poll(() => gameFrames(two)).toContainEqual({
      type: 'leftRoom', roomId: joined.room.roomId, sequence: leftSequence,
    })
    const vacancy = await roomFrame(one, joined.room.revision)
    expect(vacancy.room.seats.map((seat) => seat.occupied)).toEqual([true, false])
    await sendGame(third, { type: 'joinRoom', roomCode: created.room.roomCode })
    const replacement = await roomFrame(third)
    expect(replacement.youSeat).toBe('two')
    expect(replacement.room.roomId).toBe(created.room.roomId)

    await sendGame(two, { type: 'createRoom' })
    await expect.poll(async () => (await gameFrames(two)).some((message) =>
      message.type === 'roomState' && message.room.roomId !== created.room.roomId,
    )).toBe(true)
    const nextRoom = await roomFrame(two)
    expect(nextRoom.youSeat).toBe('one')
    expect(nextRoom.room.roomId).not.toBe(created.room.roomId)
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
