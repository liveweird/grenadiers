import { expect, it, vi } from 'vitest'
import { MAX_SERVER_FRAME_BYTES } from './clientProtocol'
import { harness } from './testSupport'

it('rejects an otherwise valid server frame above the v17 byte cap', async () => {
  const value = harness()
  value.client.admit({ type: 'create' })
  await vi.waitFor(() => expect(value.sockets).toHaveLength(1))
  const socket = value.sockets[0]!
  socket.emit('open')
  const authenticated = JSON.stringify({ type: 'authenticated', protocolVersion: 18 })
  socket.emit('message', `${authenticated}${' '.repeat(MAX_SERVER_FRAME_BYTES)}`)
  expect(value.client.getState()).toMatchObject({
    connection: 'disconnected', failure: 'invalidServerMessage',
  })
})
