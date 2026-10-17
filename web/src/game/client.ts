import Ajv from 'ajv'
import gameplaySchema from '../../../api/gameplay.schema.json'
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

const initialState = (): GameClientState => ({
  connection: 'disconnected',
  room: null,
  snapshot: null,
  youSeat: null,
  failure: null,
  admissionPending: false,
  leavePending: false,
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

export class GameClient {
  private state = initialState()
  private generation = 0
  private controller: AbortController | null = null
  private socket: WebSocket | null = null
  private nextSequence = 1
  private nextAction = 1
  private admission: AdmissionIntent | null = null
  private admissionSequence: number | null = null
  private leaveSequence: number | null = null
  private readonly dependencies: Dependencies
  private readonly onChange: (state: GameClientState) => void

  constructor(
    onChange: (state: GameClientState) => void,
    dependencies?: Partial<Dependencies>,
  ) {
    this.onChange = onChange
    this.dependencies = {
      fetch: dependencies?.fetch ?? globalThis.fetch.bind(globalThis),
      socket: dependencies?.socket ?? ((url) => new WebSocket(url)),
      location: dependencies?.location ?? window.location,
    }
  }

  getState(): GameClientState {
    return this.state
  }

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
    const room = this.state.room
    if (this.state.leavePending || !room || !['waiting', 'countdown'].includes(room.phase.type)) return false
    return this.sendRoomCommand({ type: 'ready', ready })
  }

  movement(direction: Direction | null): boolean {
    if (!this.canControl()) return false
    return this.sendRoomCommand({ type: 'movement', direction })
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
    const sent = this.send({ type: 'leaveRoom', sequence, roomId: room.roomId, roundId: room.roundId })
    if (!sent) return false
    this.leaveSequence = sequence
    this.update({ leavePending: true, failure: null })
    return true
  }

  disconnect(): void {
    this.reset(true)
  }

  dispose(): void {
    this.reset(false)
  }

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
    this.clearMembership()
    this.state = { ...initialState(), failure: this.state.failure }
    if (notify) this.emit()
  }

  private async connect(): Promise<void> {
    const generation = ++this.generation
    const controller = new AbortController()
    this.controller = controller
    this.update({ connection: 'connecting', failure: null })
    try {
      const response = await this.dependencies.fetch('/api/v1/game/session', {
        method: 'POST',
        headers: { Accept: 'application/json' },
        signal: controller.signal,
      })
      if (generation !== this.generation) return
      if (!response.ok) {
        this.fail('sessionFailed')
        return
      }
      const body: unknown = await response.json()
      if (generation !== this.generation) return
      const token = sessionToken(body)
      if (!token) {
        this.fail('sessionFailed')
        return
      }
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
      socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 2, token: credential }))
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
      if (!validateServerMessage(parsed)) {
        this.fail('invalidServerMessage', true)
        return
      }
      this.handle(parsed)
    } catch {
      this.fail('invalidServerMessage', true)
    }
  }

  private handle(message: GameServerMessage): void {
    switch (message.type) {
      case 'authenticated':
        if (this.state.connection !== 'authenticating') return
        this.update({ connection: 'authenticated', failure: null })
        this.sendAdmission()
        break
      case 'incompatibleProtocol':
        this.fail('incompatibleProtocol', true)
        break
      case 'roomState':
        this.acceptRoom(message.room, message.youSeat)
        break
      case 'snapshot':
        this.acceptSnapshot(message)
        break
      case 'leftRoom':
        this.acceptLeft(message.roomId, message.sequence)
        break
      case 'error':
        this.acceptError(message.code, message.sequence)
        break
    }
  }

  private acceptRoom(room: RoomView, youSeat: Seat): void {
    const current = this.state.room
    if (current) {
      if (room.roomId !== current.roomId || room.revision <= current.revision) return
      if (youSeat !== this.state.youSeat) return
      if (room.roundId !== current.roundId) this.nextAction = 1
    } else if (
      this.state.connection !== 'authenticated'
      || !this.admission
      || this.admissionSequence === null
    ) return
    this.admission = null
    this.admissionSequence = null
    this.update({ room, youSeat, snapshot: null, admissionPending: false })
  }

  private acceptSnapshot(message: Snapshot): void {
    const current = this.state.room
    if (!current || !this.state.youSeat) return
    if (message.room.roomId !== current.roomId || message.room.roundId !== current.roundId) return
    if (message.room.revision <= current.revision) return
    this.update({ room: message.room, snapshot: message })
  }

  private acceptLeft(roomId: string, sequence: number): void {
    if (sequence !== this.leaveSequence || roomId !== this.state.room?.roomId) return
    this.leaveSequence = null
    this.clearMembership()
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
    const admission = this.admission
    if (!admission || this.state.connection !== 'authenticated') return false
    const sequence = this.sequence()
    if (sequence === null) {
      this.admission = null
      this.update({ admissionPending: false })
      return false
    }
    const message: GameClientMessage = admission.type === 'create'
      ? { type: 'createRoom', sequence }
      : { type: 'joinRoom', sequence, roomCode: admission.roomCode }
    if (!this.send(message)) return false
    this.admissionSequence = sequence
    return true
  }

  private sendRoomCommand(command: { type: 'ready'; ready: boolean } | { type: 'movement'; direction: Direction | null } | { type: 'placeBomb'; actionId: number }): boolean {
    const room = this.state.room
    const sequence = this.sequence()
    if (!room || sequence === null) return false
    return this.send({ ...command, sequence, roomId: room.roomId, roundId: room.roundId } as GameClientMessage)
  }

  private sequence(): number | null {
    if (this.nextSequence > MAX_COUNTER) {
      this.update({ failure: 'sequenceExhausted' })
      return null
    }
    const sequence = this.nextSequence
    this.nextSequence += 1
    return sequence
  }

  private canControl(): boolean {
    if (
      this.state.connection !== 'authenticated'
      || this.state.leavePending
      || this.state.room?.phase.type !== 'running'
    ) return false
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
    this.clearMembership()
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
      return
    }
    if (!this.socket) {
      this.admission = null
      this.admissionSequence = null
      this.update({ connection: 'disconnected', admissionPending: false })
    }
  }

  private clearMembership(): void {
    this.nextAction = 1
  }

  private current(generation: number, socket: WebSocket): boolean {
    return generation === this.generation && socket === this.socket
  }

  private update(change: Partial<GameClientState>): void {
    this.state = { ...this.state, ...change }
    this.emit()
  }

  private emit(): void {
    this.onChange(this.state)
  }
}
