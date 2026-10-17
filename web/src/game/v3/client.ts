import Ajv from 'ajv'
import gameplaySchema from '../../../../api/gameplay-v3.schema.json'
import type {
  Direction,
  ErrorCode,
  GameClientMessage,
  GameServerMessage,
  RoomView,
  Seat,
  Snapshot,
} from './schema'

const MAX_COUNTER = 2_147_483_647
const TOKEN_PATTERN = /^[0-9a-f]{64}$/
const ajv = new Ajv({ allErrors: true })
const validateServerMessage = ajv.compile<GameServerMessage>(gameplaySchema)

type GameConnection = 'disconnected' | 'connecting' | 'authenticating' | 'authenticated'
type LocalGameFailure = 'connectionFailed' | 'invalidServerMessage' | 'sessionFailed' | 'sequenceExhausted'
export type GameFailure = ErrorCode | LocalGameFailure | 'incompatibleProtocol'
export type AdmissionIntent = { type: 'create' } | { type: 'join'; roomCode: string }

export interface GameClientState {
  connection: GameConnection
  room: RoomView | null
  snapshot: Snapshot | null
  youSeat: Seat | null
  failure: GameFailure | null
  admissionPending: boolean
  leavePending: boolean
}

interface Dependencies {
  fetch: typeof fetch
  socket: (url: string) => WebSocket
  location: Location
}

export const initialGameClientState = (): GameClientState => ({
  connection: 'disconnected', room: null, snapshot: null, youSeat: null,
  failure: null, admissionPending: false, leavePending: false,
})

export function gameplayUrl(location: Location = window.location): string {
  const protocol = location.protocol === 'https:' ? 'wss:' : 'ws:'
  return `${protocol}//${location.host}/api/v1/game/ws`
}

function sessionToken(value: unknown): string | null {
  if (value === null || typeof value !== 'object' || Array.isArray(value)) return null
  if (Object.keys(value).length !== 1 || !Object.hasOwn(value, 'token')) return null
  const token = Reflect.get(value, 'token')
  return typeof token === 'string' && TOKEN_PATTERN.test(token) ? token : null
}

type RoomCommand =
  | { type: 'ready'; ready: boolean }
  | { type: 'movement'; direction: Direction | null }
  | { type: 'placeBomb'; actionId: number }
  | { type: 'rematch'; ready: boolean }

export class GameClient {
  private state = initialGameClientState()
  private generation = 0
  private controller: AbortController | null = null
  private socket: WebSocket | null = null
  private nextSequence = 1
  private nextAction = 1
  private admission: AdmissionIntent | null = null
  private admissionSequence: number | null = null
  private leaveSequence: number | null = null
  private readonly onChange: (state: GameClientState) => void
  private readonly dependencies: Dependencies

  constructor(
    onChange: (state: GameClientState) => void,
    dependencies: Dependencies = {
      fetch: globalThis.fetch.bind(globalThis),
      socket: (url) => new WebSocket(url),
      location: window.location,
    },
  ) {
    this.onChange = onChange
    this.dependencies = dependencies
  }

  getState(): GameClientState { return this.state }

  admit(intent: AdmissionIntent): boolean {
    if (this.state.room || this.admission || this.state.leavePending) return false
    this.admission = intent
    this.update({ admissionPending: true, failure: null })
    if (this.state.connection === 'authenticated') return this.sendAdmission()
    if (this.state.connection !== 'disconnected') return true
    void this.connect()
    return true
  }

  ready(ready: boolean): boolean {
    const phase = this.state.room?.phase
    const initial = phase?.type === 'waiting'
      || (phase?.type === 'countdown' && phase.kind === 'initial')
    return Boolean(initial && !this.state.leavePending && this.sendRoomCommand({ type: 'ready', ready }))
  }

  rematch(ready: boolean): boolean {
    const room = this.state.room
    if (!room || room.phase.type !== 'matchResult' || this.state.leavePending) return false
    const occupied = room.seats.every((seat) => seat.occupied)
    if (!occupied || room.tick < room.phase.rematchAt) return false
    return this.sendRoomCommand({ type: 'rematch', ready })
  }

  movement(direction: Direction | null): boolean {
    return this.canControl() && this.sendRoomCommand({ type: 'movement', direction })
  }

  placeBomb(): boolean {
    if (!this.canControl() || this.nextAction > MAX_COUNTER) {
      if (this.nextAction > MAX_COUNTER) this.update({ failure: 'sequenceExhausted' })
      return false
    }
    const actionId = this.nextAction
    if (!this.sendRoomCommand({ type: 'placeBomb', actionId })) return false
    this.nextAction += 1
    return true
  }

  leave(): boolean {
    const room = this.state.room
    if (!room || this.leaveSequence !== null) return false
    const sequence = this.sequence()
    if (sequence === null) return false
    if (!this.send({ type: 'leaveRoom', sequence, roomId: room.roomId, roundId: room.roundId })) return false
    this.leaveSequence = sequence
    this.update({ leavePending: true, failure: null })
    return true
  }

  disconnect(): void { this.reset(true) }
  dispose(): void { this.reset(false) }

  private reset(notify: boolean): void {
    this.generation += 1
    this.controller?.abort()
    this.controller = null
    const socket = this.socket
    this.socket = null
    socket?.close()
    this.admission = null
    this.admissionSequence = null
    this.leaveSequence = null
    this.nextSequence = 1
    this.nextAction = 1
    this.state = { ...initialGameClientState(), failure: this.state.failure }
    if (notify) this.emit()
  }

  private async connect(): Promise<void> {
    const generation = ++this.generation
    const controller = new AbortController()
    this.controller = controller
    this.update({ connection: 'connecting', failure: null })
    try {
      const response = await this.dependencies.fetch('/api/v1/game/session', {
        method: 'POST', headers: { Accept: 'application/json' }, signal: controller.signal,
      })
      if (generation !== this.generation) return
      if (!response.ok) { this.fail('sessionFailed'); return }
      const token = sessionToken(await response.json())
      if (generation !== this.generation) return
      if (!token) { this.fail('sessionFailed'); return }
      this.openSocket(generation, token)
    } catch {
      if (generation === this.generation && !controller.signal.aborted) this.fail('sessionFailed')
    }
  }

  private openSocket(generation: number, token: string): void {
    const socket = this.dependencies.socket(gameplayUrl(this.dependencies.location))
    this.socket = socket
    let credential = token
    socket.addEventListener('open', () => {
      if (!this.current(generation, socket)) return
      this.update({ connection: 'authenticating' })
      socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 3, token: credential }))
      credential = ''
    })
    socket.addEventListener('message', (event) => this.receive(generation, socket, event.data))
    socket.addEventListener('error', () => this.closed(generation, socket))
    socket.addEventListener('close', () => this.closed(generation, socket))
  }

  private receive(generation: number, socket: WebSocket, raw: unknown): void {
    if (!this.current(generation, socket)) return
    try {
      const parsed: unknown = typeof raw === 'string' ? JSON.parse(raw) : undefined
      if (!validateServerMessage(parsed)) { this.fail('invalidServerMessage', true); return }
      this.handle(parsed)
    } catch { this.fail('invalidServerMessage', true) }
  }

  private handle(message: GameServerMessage): void {
    switch (message.type) {
      case 'authenticated':
        if (this.state.connection !== 'authenticating') return
        this.update({ connection: 'authenticated', failure: null })
        this.sendAdmission()
        break
      case 'incompatibleProtocol': this.fail('incompatibleProtocol', true); break
      case 'roomState': this.acceptRoom(message.room, message.youSeat); break
      case 'snapshot': this.acceptSnapshot(message); break
      case 'leftRoom': this.acceptLeft(message.roomId, message.sequence); break
      case 'error': this.acceptError(message.code, message.sequence); break
    }
  }

  private acceptRoom(room: RoomView, youSeat: Seat): void {
    const current = this.state.room
    if (current) {
      if (!this.newerRoom(room, current) || youSeat !== this.state.youSeat) return
      const epochChanged = room.roundId > current.roundId
      if (epochChanged) this.nextAction = 1
      this.update({ room, snapshot: epochChanged ? null : this.state.snapshot })
      return
    }
    if (this.state.connection !== 'authenticated' || !this.admission || this.admissionSequence === null) return
    this.admission = null
    this.admissionSequence = null
    this.update({ room, youSeat, snapshot: null, admissionPending: false })
  }

  private acceptSnapshot(message: Snapshot): void {
    const current = this.state.room
    if (!current || !this.state.youSeat || !this.newerRoom(message.room, current)) return
    if (message.room.roundId > current.roundId) this.nextAction = 1
    this.update({ room: message.room, snapshot: message })
  }

  private newerRoom(next: RoomView, current: RoomView): boolean {
    return next.roomId === current.roomId
      && next.revision > current.revision
      && next.roundId >= current.roundId
  }

  private acceptLeft(roomId: string, sequence: number): void {
    if (sequence !== this.leaveSequence || roomId !== this.state.room?.roomId) return
    this.leaveSequence = null
    this.nextAction = 1
    this.update({ room: null, snapshot: null, youSeat: null, leavePending: false, failure: null })
  }

  private acceptError(code: ErrorCode, sequence: number | null): void {
    if (sequence === this.admissionSequence) {
      this.admission = null
      this.admissionSequence = null
      this.update({ admissionPending: false })
    }
    if (sequence === this.leaveSequence) {
      this.leaveSequence = null
      this.update({ leavePending: false })
    }
    this.update({ failure: code })
  }

  private sendAdmission(): boolean {
    if (!this.admission || this.state.connection !== 'authenticated') return false
    const sequence = this.sequence()
    if (sequence === null) {
      this.admission = null
      this.update({ admissionPending: false })
      return false
    }
    const message: GameClientMessage = this.admission.type === 'create'
      ? { type: 'createRoom', sequence }
      : { type: 'joinRoom', sequence, roomCode: this.admission.roomCode }
    if (!this.send(message)) return false
    this.admissionSequence = sequence
    return true
  }

  private sendRoomCommand(command: RoomCommand): boolean {
    const room = this.state.room
    const sequence = this.sequence()
    if (!room || sequence === null) return false
    return this.send({ ...command, sequence, roomId: room.roomId, roundId: room.roundId } as GameClientMessage)
  }

  private sequence(): number | null {
    if (this.nextSequence > MAX_COUNTER) { this.update({ failure: 'sequenceExhausted' }); return null }
    return this.nextSequence++
  }

  private canControl(): boolean {
    if (this.state.connection !== 'authenticated' || this.state.leavePending || this.state.room?.phase.type !== 'running') return false
    const seat = this.state.youSeat
    return Boolean(seat && this.state.snapshot?.state.actors.find((actor) => actor.seat === seat)?.alive)
  }

  private send(message: GameClientMessage): boolean {
    if (this.state.connection !== 'authenticated' || this.socket?.readyState !== WebSocket.OPEN) return false
    this.socket.send(JSON.stringify(message))
    return true
  }

  private closed(generation: number, socket: WebSocket): void {
    if (!this.current(generation, socket)) return
    this.socket = null
    this.controller = null
    this.admission = null
    this.admissionSequence = null
    this.leaveSequence = null
    this.nextSequence = 1
    this.nextAction = 1
    this.update({
      connection: 'disconnected', room: null, snapshot: null, youSeat: null,
      admissionPending: false, leavePending: false,
      failure: this.state.failure ?? 'connectionFailed',
    })
  }

  private fail(failure: GameFailure, close = false): void {
    this.update({ failure })
    if (close && this.socket) {
      const socket = this.socket
      socket.close()
      this.closed(this.generation, socket)
    } else if (!this.socket) {
      this.admission = null
      this.admissionSequence = null
      this.update({ connection: 'disconnected', admissionPending: false })
    }
  }

  private current(generation: number, socket: WebSocket): boolean {
    return generation === this.generation && socket === this.socket
  }

  private update(change: Partial<GameClientState>): void {
    this.state = { ...this.state, ...change }
    this.emit()
  }

  private emit(): void { this.onChange(this.state) }
}
