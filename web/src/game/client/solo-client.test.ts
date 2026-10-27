import { expect, it } from 'vitest'
import type { Difficulty } from '../v13/schema'
import { admitted, authenticate, harness, messages, room } from './testSupport'

it.each(['easy', 'standard'] as const)(
  'creates %s solo explicitly and retains human authority with a matching server controller',
  async (difficulty: Difficulty) => {
  const value = harness()
  const socket = await authenticate(value, { type: 'create', mode: { type: 'solo', difficulty } })
  expect(messages(socket).at(-1)).toEqual({
    type: 'createRoom', sequence: 1, mode: { type: 'solo', difficulty },
  })
  const current = room(2)
  current.mode = { type: 'solo', difficulty }
  current.seats[1] = {
    seat: 'two', controller: { type: 'cpu', difficulty },
    occupied: true, ready: true, rematchReady: false, connection: { type: 'serverControlled' },
  }
  socket.message({ ...admitted(), room: current, state: null })
  expect(value.client.getState().failure).toBeNull()
  expect(value.client.updateSettings({ ...current.settings, bombFuseTicks: 60 })).toBe(true)
  current.settings = { ...current.settings, bombFuseTicks: 60 }
  current.settingsRevision += 1
  current.revision += 1
  current.tick += 1
  socket.message({ type: 'roomState', room: current })
  expect(value.client.ready(true)).toBe(true)
  expect(messages(socket).at(-1)).toMatchObject({ type: 'ready', ready: true, settingsRevision: 2 })
})

it('rejects personal admission to the CPU seat before exposing controls or retaining its capability', async () => {
  const value = harness()
  const socket = await authenticate(value)
  const frame = admitted()
  frame.room.mode = { type: 'solo', difficulty: 'standard' }
  frame.room.seats[1] = {
    seat: 'two', controller: { type: 'cpu', difficulty: 'standard' },
    occupied: true, ready: false, rematchReady: false, connection: { type: 'serverControlled' },
  }
  socket.message({ ...frame, youSeat: 'two' })
  expect(value.client.getState().failure).toBe('invalidServerMessage')
  expect(value.client.getState().room).toBeNull()
  expect(value.client.placeBomb()).toBe(false)
})
