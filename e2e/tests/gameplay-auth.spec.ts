import { expect, test } from '@playwright/test'

test('gameplay authentication preserves credentials and connection ownership', async ({ page }) => {
  await page.goto('/')
  const frames = await page.evaluate(async () => {
    const response = await fetch('/api/v1/game/session', { method: 'POST' })
    const session: unknown = await response.json()
    if (response.status !== 201 || typeof session !== 'object' || session === null
      || !('token' in session) || typeof session.token !== 'string') {
      throw new Error('Session issuance failed')
    }
    const token = session.token
    const endpoint = new URL('/api/v1/game/ws', location.href)
    endpoint.protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
    const sockets: WebSocket[] = []

    function nextFrame(socket: WebSocket): Promise<unknown> {
      return new Promise((resolve, reject) => {
        const timer = setTimeout(() => fail(), 10_000)
        function cleanup(): void {
          clearTimeout(timer)
          socket.removeEventListener('message', message)
          socket.removeEventListener('error', fail)
          socket.removeEventListener('close', fail)
        }
        function fail(): void {
          cleanup()
          reject(new Error('Gameplay response unavailable'))
        }
        function message(event: MessageEvent): void {
          cleanup()
          if (typeof event.data !== 'string') {
            reject(new Error('Unexpected gameplay frame'))
            return
          }
          try {
            const value: unknown = JSON.parse(event.data)
            resolve(value)
          } catch {
            reject(new Error('Invalid gameplay response'))
          }
        }
        socket.addEventListener('message', message)
        socket.addEventListener('error', fail)
        socket.addEventListener('close', fail)
      })
    }

    async function connect(protocolVersion: number): Promise<{ socket: WebSocket; frame: unknown }> {
      const socket = new WebSocket(endpoint)
      sockets.push(socket)
      const frame = nextFrame(socket)
      socket.addEventListener('open', () => {
        socket.send(JSON.stringify({ type: 'authenticate', protocolVersion, token }))
      }, { once: true })
      return { socket, frame: await frame }
    }

    try {
      const incompatible = await connect(12)
      const owner = await connect(13)
      const duplicate = await connect(13)
      const command = {
        type: 'movement', sequence: 1, roomId: 'a'.repeat(32), roundId: 1, direction: null,
      }
      const ownerResponse = nextFrame(owner.socket)
      owner.socket.send(JSON.stringify(command))
      const firstCommand = await ownerResponse
      const replayResponse = nextFrame(owner.socket)
      owner.socket.send(JSON.stringify(command))
      return {
        incompatible: incompatible.frame,
        authenticated: owner.frame,
        duplicate: duplicate.frame,
        firstCommand,
        replay: await replayResponse,
      }
    } finally {
      for (const socket of sockets) socket.close()
    }
  })

  expect(frames.incompatible).toEqual({ type: 'incompatibleProtocol', supportedVersion: 13 })
  expect(frames.authenticated).toEqual({ type: 'authenticated', protocolVersion: 13 })
  expect(frames.duplicate).toEqual({ type: 'error', code: 'connectionInUse', sequence: null })
  expect(frames.firstCommand).toEqual({ type: 'error', code: 'notInRoom', sequence: 1 })
  expect(frames.replay).toEqual({ type: 'error', code: 'staleSequence', sequence: 1 })
})
