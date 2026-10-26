import { beforeEach, describe, expect, it, vi } from 'vitest'
import { admitted, authenticate, harness, messages, room, snapshot, SocketStub } from './testSupport'

describe('v12 phase-boundary control errors', () => {
  beforeEach(() => vi.stubGlobal('WebSocket', { OPEN: SocketStub.OPEN }))

  it.each(['invalidPhase', 'staleRoom'] as const)(
    'ignores a correlated %s reply after authoritative phase progress',
    async (code) => {
      const value = harness()
      const socket = await authenticate(value)
      socket.message(admitted())
      expect(value.client.movement('east')).toBe(true)
      const movement = messages(socket).at(-1) as { sequence: number }

      socket.message({ type: 'roomState', room: room(2, { type: 'result', holdUntil: 181 }) })
      socket.message({ type: 'error', code, sequence: movement.sequence })

      expect(value.client.getState().failure).toBeNull()
    },
  )

  it('bounds movement correlation context to the latest 64 commands', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())

    for (let index = 0; index < 70; index += 1) {
      expect(value.client.movement('north')).toBe(true)
    }

    const controls = Reflect.get(value.client, 'sentControls') as Array<{ sequence: number }>
    expect(controls).toHaveLength(64)
    expect(controls[0].sequence).toBe(8)
    expect(controls.at(-1)?.sequence).toBe(71)
  })

  it('keeps a current-phase movement error visible through newer running state', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    expect(value.client.movement('north')).toBe(true)
    const movement = messages(socket).at(-1) as { sequence: number }

    socket.message({ type: 'error', code: 'invalidPhase', sequence: movement.sequence })
    socket.message(snapshot(2))

    expect(value.client.getState().failure).toBe('invalidPhase')
  })

  it('does not clear non-movement action errors after phase progress', async () => {
    const value = harness()
    const socket = await authenticate(value)
    socket.message(admitted())
    expect(value.client.placeBomb()).toBe(true)
    const action = messages(socket).at(-1) as { sequence: number }

    socket.message({ type: 'error', code: 'invalidPhase', sequence: action.sequence })
    socket.message({ type: 'roomState', room: room(2, { type: 'result', holdUntil: 181 }) })
    expect(value.client.getState().failure).toBe('invalidPhase')

    socket.message({ type: 'error', code: 'duplicateAction', sequence: action.sequence })
    socket.message({ type: 'roomState', room: room(3, { type: 'result', holdUntil: 182 }) })
    expect(value.client.getState().failure).toBe('duplicateAction')
  })
})
