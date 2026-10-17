import { MantineProvider } from '@mantine/core'
import { act, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameArenaProps } from './game/arena/types'
import type { RoomPhase, RoomView, Snapshot } from './game/schema'
import i18n from './i18n'
import { App } from './App'

vi.mock('./game/arena/GameArena', () => ({
  GameArena: ({ label, waitingText, enabled, onDirection, onBomb }: GameArenaProps) => (
    <div role="region" aria-label={label} data-enabled={String(enabled)}>
      {waitingText}
      {enabled && <button onClick={() => onDirection('north')}>Test move</button>}
      {enabled && <button onClick={onBomb}>Test bomb</button>}
    </div>
  ),
}))

const token = 'a'.repeat(64)
const roomId = 'b'.repeat(32)

class AppSocketStub {
  static readonly OPEN = 1
  readonly listeners = new Map<string, Array<(event: MessageEvent) => void>>()
  readonly sent: string[] = []
  readonly close = vi.fn()
  readonly url: string
  readyState = 0

  constructor(url: string) { this.url = url; appSockets.push(this) }
  addEventListener(name: string, listener: (event: MessageEvent) => void): void {
    this.listeners.set(name, [...(this.listeners.get(name) ?? []), listener])
  }
  send(message: string): void { this.sent.push(message) }
  emit(name: string, data?: string): void {
    if (name === 'open') this.readyState = AppSocketStub.OPEN
    for (const listener of this.listeners.get(name) ?? []) listener({ data } as MessageEvent)
  }
  message(message: object): void { this.emit('message', JSON.stringify(message)) }
}

const appSockets: AppSocketStub[] = []

function room(revision: number, phase: RoomPhase = { type: 'waiting' }): RoomView {
  return {
    roomId,
    roomCode: 'ABCDEFGH',
    roundId: 1,
    revision,
    tick: revision,
    rulesVersion: 1,
    mapId: 'fixed_v1',
    seats: [
      { seat: 'one', occupied: true, ready: false },
      { seat: 'two', occupied: true, ready: false },
    ],
    phase,
  }
}

function snapshot(revision: number, phase: RoomPhase = { type: 'running' }): Snapshot {
  return {
    type: 'snapshot',
    room: room(revision, phase),
    state: {
      runningUpdates: 1,
      remainingTicks: 10_800,
      arena: { width: 15, height: 13, terrain: Array.from({ length: 195 }, () => 'floor'), blocks: [] },
      actors: [
        {
          id: 1, seat: 'one', position: { x: 512, y: 512 }, heading: 'east', speedLevel: 0,
          alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        },
        {
          id: 2, seat: 'two', position: { x: 14_848, y: 12_800 }, heading: 'west', speedLevel: 0,
          alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
        },
      ],
      bombs: [],
      flames: [],
      pickups: [],
    },
    events: [],
  }
}

function renderApp() {
  return render(<MantineProvider env="test"><App /></MantineProvider>)
}

async function beginRoom(user: ReturnType<typeof userEvent.setup>, action: 'create' | 'join' = 'create') {
  if (action === 'create') await user.click(screen.getByRole('button', { name: 'Create room' }))
  else {
    await user.type(screen.getByRole('textbox', { name: 'Room code' }), 'abcdefgh')
    await user.click(screen.getByRole('button', { name: 'Join room' }))
  }
  await waitFor(() => expect(appSockets.filter((socket) => socket.url.endsWith('/api/v1/game/ws'))).toHaveLength(1))
  const socket = appSockets.find((candidate) => candidate.url.endsWith('/api/v1/game/ws'))!
  act(() => socket.emit('open'))
  expect(JSON.parse(socket.sent[0])).toEqual({ type: 'authenticate', protocolVersion: 2, token })
  act(() => socket.message({ type: 'authenticated', protocolVersion: 2 }))
  return socket
}

describe('App game flow', () => {
  beforeEach(async () => {
    appSockets.length = 0
    vi.stubGlobal('fetch', vi.fn(async (input: RequestInfo | URL) => ({
      ok: true,
      json: async () => String(input).endsWith('/api/v1/health') ? { status: 'ok' } : { token },
    } as Response)))
    vi.stubGlobal('WebSocket', AppSocketStub)
    await i18n.changeLanguage('en')
  })

  it('creates a room, reports player readiness, and clears only the correlated leave', async () => {
    const user = userEvent.setup()
    renderApp()
    expect(screen.getByRole('heading', { name: 'Grenadiers' })).toBeVisible()
    expect(screen.getByText('Clear a path. Survive the round.')).toBeVisible()
    const socket = await beginRoom(user)
    expect(JSON.parse(socket.sent[1])).toEqual({ type: 'createRoom', sequence: 1 })

    act(() => socket.message({ type: 'roomState', room: room(1), youSeat: 'one' }))
    expect(await screen.findByLabelText('Room invitation code')).toHaveTextContent('ABCDEFGH')
    expect(screen.getAllByText('Player one')).toHaveLength(2)
    await user.click(screen.getByRole('button', { name: 'Ready' }))
    expect(JSON.parse(socket.sent[2])).toEqual({
      type: 'ready', ready: true, sequence: 2, roomId, roundId: 1,
    })
    act(() => socket.message({
      type: 'roomState', room: room(2, { type: 'countdown', endsAt: 182 }), youSeat: 'one',
    }))
    expect(screen.getByLabelText('Round time remaining')).toHaveTextContent('0:03')
    expect(screen.getAllByText('Not ready')).toHaveLength(2)

    await user.click(screen.getByRole('button', { name: 'Leave room' }))
    const leave = JSON.parse(socket.sent[3]) as { sequence: number }
    expect(screen.queryByRole('button', { name: 'Ready' })).not.toBeInTheDocument()
    act(() => socket.message({ type: 'leftRoom', roomId, sequence: leave.sequence + 1 }))
    expect(screen.getByLabelText('Room invitation code')).toBeVisible()
    act(() => socket.message({ type: 'leftRoom', roomId, sequence: leave.sequence }))
    expect(await screen.findByRole('button', { name: 'Create room' })).toBeVisible()
  })

  it('joins canonical codes, keeps the socket through locale changes, and localizes typed errors', async () => {
    const user = userEvent.setup()
    renderApp()
    await user.type(screen.getByRole('textbox', { name: 'Room code' }), 'bad')
    expect(screen.getByRole('button', { name: 'Join room' })).toBeDisabled()
    await user.clear(screen.getByRole('textbox', { name: 'Room code' }))
    const socket = await beginRoom(user, 'join')
    expect(JSON.parse(socket.sent[1])).toEqual({ type: 'joinRoom', sequence: 1, roomCode: 'ABCDEFGH' })

    act(() => socket.message({ type: 'error', code: 'roomFull', sequence: 1 }))
    expect(await screen.findByText('Both seats in that room are occupied.')).toBeVisible()
    await user.click(screen.getByRole('radio', { name: 'Polski' }))
    expect(await screen.findByRole('button', { name: 'Utwórz pokój' })).toBeVisible()
    expect(screen.getByText('Oba miejsca w tym pokoju są zajęte.')).toBeVisible()
    expect(socket.close).not.toHaveBeenCalled()
  })

  it('shows authoritative play, timer, and result states', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => socket.message({ type: 'roomState', room: room(1, { type: 'running' }), youSeat: 'one' }))
    act(() => socket.message(snapshot(2)))
    expect(await screen.findByLabelText('Round time remaining')).toHaveTextContent('3:00')
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-enabled', 'true')
    await user.click(screen.getByRole('button', { name: 'Test move' }))
    await user.click(screen.getByRole('button', { name: 'Test bomb' }))
    expect(JSON.parse(socket.sent.at(-2)!)).toMatchObject({ type: 'movement', direction: 'north' })
    expect(JSON.parse(socket.sent.at(-1)!)).toMatchObject({ type: 'placeBomb', actionId: 1 })

    await user.click(screen.getByRole('button', { name: 'Leave room' }))
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-enabled', 'false')
    expect(screen.queryByRole('button', { name: 'Test move' })).not.toBeInTheDocument()

    act(() => socket.message(snapshot(3, {
      type: 'result', endedAt: 3, result: { type: 'draw', reason: 'timeout' },
    })))
    expect(await screen.findByText('Draw: time ran out.')).toBeVisible()
  })

  it('freezes the authoritative remaining time when elimination ends a round', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => socket.message({ type: 'roomState', room: room(1, { type: 'running' }), youSeat: 'one' }))
    const finished = snapshot(2, {
      type: 'result', endedAt: 300, result: { type: 'winner', seat: 'two' },
    })
    finished.state.runningUpdates = 300
    finished.state.remainingTicks = 10_500
    finished.state.actors[0].alive = false
    finished.room.tick = 300
    act(() => socket.message(finished))
    expect(await screen.findByText('Player two wins the round.')).toBeVisible()
    expect(screen.getByLabelText('Round time remaining')).toHaveTextContent('2:55')
    act(() => socket.message({ ...finished, room: { ...finished.room, revision: 3, tick: 303 } }))
    expect(screen.getByLabelText('Round time remaining')).toHaveTextContent('2:55')
  })

  it('shows a pre-start interruption without inventing a winner', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => socket.message({
      type: 'roomState',
      room: room(1, { type: 'interrupted', endedAt: 1, reason: 'connectionLost' }),
      youSeat: 'one',
    }))
    expect(await screen.findByText('The other player lost their connection.')).toBeVisible()
    expect(screen.queryByText(/wins the round/)).not.toBeInTheDocument()
  })

  it('keeps diagnostic Connect and Ping controls in their disclosure', async () => {
    const user = userEvent.setup()
    renderApp()
    await user.click(screen.getByText('Connection diagnostics'))
    expect(await screen.findByText('Available')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    const socket = appSockets.find((candidate) => candidate.url.endsWith('/api/v1/ws'))!
    act(() => socket.emit('open'))
    act(() => socket.message({ type: 'welcome', protocolVersion: 1, maxPlayers: 2 }))
    expect(await screen.findByText('Protocol v1 · up to 2 players')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Send ping' }))
    expect(socket.sent).toEqual(['{"type":"ping","sequence":1}'])
    act(() => socket.message({ type: 'pong', sequence: 1 }))
    expect(await screen.findByText('Pong 1 received')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Disconnect' }))
    expect(socket.close).toHaveBeenCalled()
  })

  it('reports diagnostic health and message failures without exposing play controls', async () => {
    vi.mocked(fetch).mockRejectedValueOnce(new Error('offline'))
    const user = userEvent.setup()
    renderApp()
    await user.click(screen.getByText('Connection diagnostics'))
    expect(await screen.findByText('Unavailable')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    const socket = appSockets.find((candidate) => candidate.url.endsWith('/api/v1/ws'))!
    act(() => socket.message({ type: 'unknown' }))
    expect(await screen.findByText('The server sent an invalid message.')).toBeVisible()
    expect(socket.close).toHaveBeenCalled()
  })

  it('aborts requests and closes both game and diagnostic sockets on cleanup', async () => {
    const user = userEvent.setup()
    const view = renderApp()
    const gameSocket = await beginRoom(user)
    await user.click(screen.getByText('Connection diagnostics'))
    await user.click(screen.getByRole('button', { name: 'Connect' }))
    const diagnostic = appSockets.find((candidate) => candidate.url.endsWith('/api/v1/ws'))!
    view.unmount()
    expect(gameSocket.close).toHaveBeenCalled()
    expect(diagnostic.close).toHaveBeenCalled()
  })
})
