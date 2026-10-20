import { MantineProvider } from '@mantine/core'
import { act, render, screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameArenaProps } from './game/arena/types'
import type { GameState, RoomPhase, RoomView, Snapshot } from './game/v8/schema'
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
const resumeToken = 'c'.repeat(64)
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
    rulesVersion: 5,
    mapId: 'generated_v2',
    roundNumber: ['running', 'result', 'matchResult'].includes(phase.type) ? 1 : 0,
    scores: { one: 0, two: phase.type === 'result' ? 1 : 0 },
    lastRound: phase.type === 'result' ? {
      roundId: 1, roundNumber: 1, endedAt: phase.holdUntil - 180,
      result: { type: 'winner', seat: 'two' },
    } : null,
    settings: {
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
      enabledItems: ['heart'],
    },
    availableItems: ['heart', 'piercingBomb', 'powerBomb', 'shield', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls'],
    settingsRevision: 1,
    settingsEditor: 'one',
    seats: [
      { seat: 'one', occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
      { seat: 'two', occupied: true, ready: false, rematchReady: false, connection: { type: 'connected' } },
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
      physicalTick: revision,
      arena: { width: 15, height: 13, terrain: Array.from({ length: 195 }, () => 'floor'), blocks: [] },
      actors: [
        {
          type: 'visible',
          id: 1, seat: 'one', position: { x: 512, y: 512 }, heading: 'east', speedLevel: 0,
          alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
          inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
        },
        {
          type: 'visible',
          id: 2, seat: 'two', position: { x: 14_848, y: 12_800 }, heading: 'west', speedLevel: 0,
          alive: true, bombCapacity: 1, bombRange: 1, liveBombCount: 0,
          inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0,
        },
      ],
      bombs: [],
      flames: [],
      pickups: [],
    },
    events: [],
  }
}

function admit(socket: AppSocketStub, currentRoom: RoomView, state: GameState | null = null) {
  socket.message({
    type: 'admitted', sequence: 1, room: currentRoom, youSeat: 'one', resumeToken,
    lastActionId: null, state,
  })
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
  expect(JSON.parse(socket.sent[0])).toEqual({ type: 'authenticate', protocolVersion: 8, token })
  act(() => socket.message({ type: 'authenticated', protocolVersion: 8 }))
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

    act(() => admit(socket, room(1)))
    expect(await screen.findByLabelText('Room invitation code')).toHaveTextContent('ABCDEFGH')
    const players = screen.getByRole('list', { name: 'Players' })
    expect(within(players).getByText('Player one')).toBeVisible()
    expect(within(players).getAllByText('Not ready')).toHaveLength(2)
    expect(screen.getByLabelText('Round wins for Player one')).toHaveTextContent('0')
    await user.click(screen.getByRole('button', { name: 'Ready' }))
    expect(JSON.parse(socket.sent[2])).toEqual({
      type: 'ready', ready: true, sequence: 2, roomId, roundId: 1, settingsRevision: 1,
    })
    const countdown = room(2, { type: 'countdown', kind: 'initial', endsAt: 182 })
    countdown.seats.forEach((seat) => { seat.ready = true })
    act(() => socket.message({ type: 'roomState', room: countdown }))
    expect(screen.getByLabelText('Phase time remaining')).toHaveTextContent('0:03')
    expect(within(players).getAllByText('Ready')).toHaveLength(2)

    await user.click(screen.getByRole('button', { name: 'Leave room' }))
    const leave = JSON.parse(socket.sent[3]) as { sequence: number }
    expect(screen.getByRole('button', { name: 'Leave room' })).toBeDisabled()
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
    expect(await screen.findByText('Both seats are occupied.')).toBeVisible()
    await user.click(screen.getByRole('radio', { name: 'Polski' }))
    expect(await screen.findByRole('button', { name: 'Utwórz pokój' })).toBeVisible()
    expect(screen.getByText('Oba miejsca są zajęte.')).toBeVisible()
    expect(socket.close).not.toHaveBeenCalled()
  })

  it('shows authoritative play, timer, and result states', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => admit(socket, room(1)))
    const live = snapshot(2)
    Object.assign(live.state.actors[0], { bombCapacity: 8, bombRange: 8, liveBombCount: 3, speedLevel: 4 })
    act(() => socket.message(live))
    expect(await screen.findByLabelText('Phase time remaining')).toHaveTextContent('3:00')
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-enabled', 'true')
    expect(screen.getAllByText('Alive')).toHaveLength(2)
    expect(screen.queryByText('Not ready')).not.toBeInTheDocument()
    const upgrades = screen.getByLabelText('Player one upgrades')
    expect(within(upgrades).getByText('Bombs available').nextElementSibling).toHaveTextContent('5')
    expect(within(upgrades).getByText('Bomb capacity').nextElementSibling).toHaveTextContent('8')
    expect(within(upgrades).getByText('Blast range').nextElementSibling).toHaveTextContent('8')
    expect(within(upgrades).getByText('Speed level').nextElementSibling).toHaveTextContent('5')
    await user.click(screen.getByRole('button', { name: 'Test move' }))
    await user.click(screen.getByRole('button', { name: 'Test bomb' }))
    expect(JSON.parse(socket.sent.at(-2)!)).toMatchObject({ type: 'movement', direction: 'north' })
    expect(JSON.parse(socket.sent.at(-1)!)).toMatchObject({
      type: 'manualAction', action: 'bomb', actionId: 1,
    })

    await user.click(screen.getByRole('button', { name: 'Leave room' }))
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-enabled', 'false')
    expect(screen.queryByRole('button', { name: 'Test move' })).not.toBeInTheDocument()

    const draw = snapshot(3, { type: 'result', holdUntil: 183 })
    draw.room.lastRound!.result = { type: 'draw', reason: 'timeout' }
    draw.room.scores.two = 0
    act(() => socket.message(draw))
    expect(await screen.findByText('Draw: time ran out.')).toBeVisible()
  })

  it('keeps frozen round physics while the authoritative result-hold clock advances', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => admit(socket, room(1)))
    const finished = snapshot(2, {
      type: 'result', holdUntil: 480,
    })
    finished.state.runningUpdates = 300
    finished.state.remainingTicks = 10_500
    finished.state.actors[0].alive = false
    finished.room.tick = 300
    act(() => socket.message(finished))
    expect(await screen.findByText('Player two wins the round.')).toBeVisible()
    expect(screen.getByText('Eliminated')).toBeVisible()
    expect(screen.getByText('Alive')).toBeVisible()
    expect(screen.getByLabelText('Phase time remaining')).toHaveTextContent('0:03')
    act(() => socket.message({ ...finished, room: { ...finished.room, revision: 3, tick: 360 } }))
    expect(screen.getByLabelText('Phase time remaining')).toHaveTextContent('0:02')
    expect(finished.state.remainingTicks).toBe(10_500)
  })

  it('localizes authoritative actor status and upgrade labels without reconnecting', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => admit(socket, room(1)))
    act(() => socket.message(snapshot(2)))
    await user.click(screen.getByRole('radio', { name: 'Polski' }))
    expect(await screen.findAllByText('W grze')).toHaveLength(2)
    const upgrades = screen.getByLabelText('Ulepszenia: Gracz pierwszy')
    expect(within(upgrades).getByText('Dostępne bomby')).toBeVisible()
    expect(within(upgrades).getByText('Limit bomb')).toBeVisible()
    expect(within(upgrades).getByText('Zasięg wybuchu')).toBeVisible()
    expect(within(upgrades).getByText('Poziom szybkości')).toBeVisible()
    expect(socket.close).not.toHaveBeenCalled()
  })

  it('shows a pre-start interruption without inventing a winner', async () => {
    const user = userEvent.setup()
    renderApp()
    const socket = await beginRoom(user)
    act(() => admit(socket, room(1, { type: 'interrupted', endedAt: 1, reason: 'connectionLost' })))
    expect(await screen.findByText('a player lost their connection')).toBeVisible()
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
