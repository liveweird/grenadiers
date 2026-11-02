import { controllersAvailable } from './roomControllers'
import type { Direction, GameClientMessage, GameServerMessage, GameSettings, RoomView, Snapshot, V14ErrorCode } from '../v20/schema'
import { initialGameClientState, type AdmissionIntent, type GameClientState, type GameFailure } from './clientTypes'
import { ClientRecovery } from './clientRecovery'
import { preservesSnapshotHistory } from './snapshotHistory'
import { requestCpuDebug } from './cpuDebugRequest'
import type { CpuDebugHttpResult } from './cpuDebugTypes'
export { initialGameClientState, type AdmissionIntent, type GameClientState, type GameFailure } from './clientTypes'
import { acceptableBaseline, canControl, controlIsObsolete, createConfiguration, dependenciesWithDefaults,
  gameplayUrl, isCurrentConnection, MAX_COUNTER, MAX_RECONNECT_ATTEMPTS, newerRoom, parseServerFrame,
  sessionToken, ownSeatConnected, settlesPendingSettings, validateServerMessage, type Capability, type ControlCommand,
  validGameSettings, type ControlFailure, type Dependencies, type PendingSettings, type RoomCommand } from './clientProtocol'
export { gameplayUrl } from './clientProtocol'
export class GameClient {
  private state = initialGameClientState()
  private generation = 0; private controller: AbortController | null = null
  private socket: WebSocket | null = null; private nextSequence = 1; private nextAction = 1
  private admission: AdmissionIntent | null = null; private capability: Capability | null = null
  private admissionSequence: number | null = null; private leaveSequence: number | null = null
  private settingsSequence: number | null = null; private pendingSettings: PendingSettings | null = null
  private sentControls: ControlCommand[] = []; private controlFailure: ControlFailure | null = null; private stopped = false
  private readonly onChange: (state: GameClientState) => void; private readonly dependencies: Dependencies; private readonly recovery: ClientRecovery
  constructor(onChange: (state: GameClientState) => void, dependencies?: Partial<Dependencies>) {
    this.onChange = onChange; this.dependencies = dependenciesWithDefaults(dependencies)
    this.recovery = new ClientRecovery(this.dependencies, () => this.terminal('connectionFailed', true))
  }
  getState(): GameClientState { return this.state }
  requestCpuDebug(signal: AbortSignal): Promise<CpuDebugHttpResult> {
    const generation = this.generation; const capability = this.capability; return requestCpuDebug(
      this.dependencies.fetch, this.state, capability,
      () => generation === this.generation && capability === this.capability, signal)
  }
  admit(intent: AdmissionIntent): boolean {
    if (this.state.room || this.admission || this.state.leavePending) return false
    this.stopped = false
    this.admission = intent
    this.update({ admissionPending: true, failure: null })
    if (this.state.connection === 'authenticated') return this.sendAdmission()
    if (this.state.connection !== 'disconnected') return true
    void this.connect(false)
    return true
  }
  ready(ready: boolean): boolean {
    const room = this.state.room
    const phase = this.state.room?.phase
    const initial = phase?.type === 'waiting' || (phase?.type === 'countdown' && phase.kind === 'initial')
    return Boolean(initial && ownSeatConnected(this.state) && !this.state.admissionPending && !this.state.leavePending
      && !this.state.settingsPending && room
      && this.sendRoomCommand({ type: 'ready', ready, settingsRevision: room.settingsRevision }))
  }
  updateSettings(settings: GameSettings): boolean {
    const room = this.state.room
    const own = room?.seats.find((seat) => seat.seat === this.state.youSeat)
    const canonicalItems = room?.availableItems.filter((item) => settings.enabledItems.includes(item))
    if (!room || !validGameSettings(settings) || settings.enabledItems.length > room.availableItems.length
      || new Set(settings.enabledItems).size !== settings.enabledItems.length
      || settings.enabledItems.some((item, index) => item !== canonicalItems?.[index])
      || settings.enabledItems.some((item) => !room.availableItems.includes(item))
      || room.phase.type !== 'waiting' || room.settingsEditor !== this.state.youSeat
      || own?.connection.type !== 'connected' || own.ready || this.state.admissionPending
      || this.state.leavePending || this.state.settingsPending) return false
    const sequence = this.sequence()
    if (sequence === null || !this.send({
      type: 'updateSettings', sequence, roomId: room.roomId, roundId: room.roundId,
      settingsRevision: room.settingsRevision, settings,
    })) return false
    this.settingsSequence = sequence
    this.pendingSettings = {
      roomId: room.roomId, roundId: room.roundId, settingsRevision: room.settingsRevision, settings,
    }
    this.update({ settingsPending: true, failure: null })
    return true
  }
  rematch(ready: boolean): boolean {
    const room = this.state.room
    if (!room || room.phase.type !== 'matchResult' || room.tick < room.phase.rematchAt) return false
    if (!controllersAvailable(room)) return false
    return !this.state.admissionPending && !this.state.leavePending
      && this.sendRoomCommand({ type: 'rematch', ready })
  }
  movement(direction: Direction | null): boolean {
    return canControl(this.state) && this.sendRoomCommand({ type: 'movement', direction }) }
  placeBomb(): boolean {
    if (!canControl(this.state) || this.nextAction > MAX_COUNTER) {
      if (this.nextAction > MAX_COUNTER) this.update({ failure: 'sequenceExhausted' })
      return false
    }
    if (!this.sendRoomCommand({ type: 'manualAction', actionId: this.nextAction, action: 'bomb' })) return false
    this.nextAction += 1
    return true
  }
  leave(): boolean {
    const room = this.state.room
    if (!room || this.state.admissionPending || this.leaveSequence !== null
      || this.state.connection !== 'authenticated') return false
    const sequence = this.sequence()
    if (sequence === null) return false
    if (!this.send({ type: 'leaveRoom', sequence, roomId: room.roomId, roundId: room.roundId })) return false
    this.leaveSequence = sequence
    this.update({ leavePending: true, failure: null })
    return true
  }
  disconnect(): void { this.reset(true) } dispose(): void { this.reset(false) }
  private reset(notify: boolean): void {
    this.stopped = true
    this.generation += 1
    this.controller?.abort()
    this.controller = null
    this.recovery.complete()
    const socket = this.socket
    this.socket = null
    socket?.close()
    this.clearMembership()
    this.state = { ...initialGameClientState(), failure: this.state.failure }
    if (notify) this.onChange(this.state)
  }
  private async connect(resume: boolean): Promise<void> {
    const generation = ++this.generation
    const controller = new AbortController()
    this.controller = controller
    this.update({ connection: resume ? 'reconnecting' : 'connecting', failure: null })
    try {
      const response = await this.dependencies.fetch('/api/v1/game/session', {
        method: 'POST', headers: { Accept: 'application/json' }, signal: controller.signal,
      })
      if (generation !== this.generation) return
      if (resume && !this.recovery.hasTime()) { this.terminal('connectionFailed', true); return }
      if (!response.ok) { this.retryOrFail(resume, 'sessionFailed'); return }
      const token = sessionToken(await response.json())
      if (generation !== this.generation) return
      if (resume && !this.recovery.hasTime()) { this.terminal('connectionFailed', true); return }
      if (!token) { this.retryOrFail(resume, 'sessionFailed'); return }
      this.openSocket(generation, token, resume)
    } catch {
      if (generation === this.generation && !controller.signal.aborted) this.retryOrFail(resume, 'sessionFailed')
    }
  }
  private openSocket(generation: number, token: string, resume: boolean): void {
    const socket = this.dependencies.socket(gameplayUrl(this.dependencies.location))
    this.socket = socket
    let credential = token
    socket.addEventListener('open', () => {
      if (!isCurrentConnection(generation, socket, this.generation, this.socket)) return
      this.update({ connection: 'authenticating' })
      socket.send(JSON.stringify({ type: 'authenticate', protocolVersion: 20, token: credential }))
      credential = ''
    })
    socket.addEventListener('message', (event) => this.receive(generation, socket, event.data))
    socket.addEventListener('error', () => this.closed(generation, socket, 0, resume))
    socket.addEventListener('close', (event) => this.closed(generation, socket, (event as CloseEvent).code, resume))
  }
  private receive(generation: number, socket: WebSocket, raw: unknown): void {
    if (!isCurrentConnection(generation, socket, this.generation, this.socket)) return
    try {
      const parsed = parseServerFrame(raw)
      if (!validateServerMessage(parsed, this.capability?.seat ?? null)) {
        this.terminal('invalidServerMessage', true); return
      }
      this.handle(parsed)
    } catch { this.terminal('invalidServerMessage', true) }
  }
  private handle(message: GameServerMessage): void {
    switch (message.type) {
      case 'authenticated':
        if (this.state.connection !== 'authenticating') return
        this.update({ connection: 'authenticated', failure: null })
        this.sendAdmission()
        break
      case 'incompatibleProtocol': this.terminal('incompatibleProtocol', true); break
      case 'admitted': this.acceptAdmitted(message); break
      case 'roomState': this.acceptRoom(message.room); break
      case 'snapshot': this.acceptSnapshot(message); break
      case 'leftRoom': this.acceptLeft(message.roomId, message.sequence); break
      case 'seatReplaced':
        if (message.roomId === this.capability?.roomId) this.terminal('seatReplaced', true)
        break
      case 'error': this.acceptError(message.code, message.sequence); break
    }
  }
  private acceptAdmitted(message: Extract<GameServerMessage, { type: 'admitted' }>): void {
    if (message.sequence !== this.admissionSequence || this.state.connection !== 'authenticated') return
    if (this.capability && !this.recovery.hasTime()) { this.terminal('connectionFailed', true); return }
    const existing = this.capability
    if (existing && (message.room.roomId !== existing.roomId || message.youSeat !== existing.seat
      || message.resumeToken !== existing.token || !acceptableBaseline(message.room, this.state.room))) return
    if (!preservesSnapshotHistory(this.state.snapshot, message.room, message.state)) {
      this.terminal('invalidServerMessage', true); return
    }
    this.capability = { roomId: message.room.roomId, seat: message.youSeat, token: message.resumeToken }
    this.admission = null
    this.nextAction = message.lastActionId === null ? 1 : message.lastActionId + 1
    this.clearPendingCommands()
    this.recovery.complete()
    this.update({
      room: message.room, youSeat: message.youSeat,
      snapshot: message.state ? { type: 'snapshot', room: message.room, state: message.state, events: [] } : null,
      admissionBaseline: {
        roomId: message.room.roomId, revision: message.room.revision, tick: message.room.tick,
        physicalTick: message.state?.physicalTick ?? null,
      },
      admissionPending: false, settingsPending: false, reconnectAttempt: 0, failure: null,
    })
  }
  private acceptRoom(room: RoomView): void {
    const current = this.state.room
    const settlesSettings = current ? settlesPendingSettings(this.pendingSettings, room, current) : false
    if (!current || !this.capability || this.state.admissionPending
      || (!newerRoom(room, current) && !settlesSettings)) return
    const epochChanged = room.roundId > current.roundId
    if (epochChanged) this.nextAction = 1
    const settingsAccepted = room.settingsRevision > current.settingsRevision || settlesSettings
    if (settingsAccepted) {
      this.settingsSequence = null
      this.pendingSettings = null
    }
    this.update({ room: settlesSettings ? current : room, snapshot: epochChanged ? null : this.state.snapshot,
      ...(settingsAccepted ? { settingsPending: false } : {}), ...this.clearObsoleteControlFailure(room) })
  }
  private acceptSnapshot(message: Snapshot): void {
    const current = this.state.room
    const settlesSettings = current ? settlesPendingSettings(this.pendingSettings, message.room, current) : false
    if (!current || !this.capability || this.state.admissionPending
      || (!newerRoom(message.room, current) && !settlesSettings)) return
    if (!settlesSettings && (!preservesSnapshotHistory(this.state.snapshot, message.room, message.state))) {
      this.terminal('invalidServerMessage', true); return
    }
    if (message.room.roundId > current.roundId) this.nextAction = 1
    const settingsAccepted = message.room.settingsRevision > current.settingsRevision || settlesSettings
    if (settingsAccepted) {
      this.settingsSequence = null
      this.pendingSettings = null
    }
    this.update({ room: settlesSettings ? current : message.room, snapshot: settlesSettings ? this.state.snapshot : message,
      ...(settingsAccepted ? { settingsPending: false } : {}), ...this.clearObsoleteControlFailure(message.room) })
  }
  private acceptLeft(roomId: string, sequence: number): void {
    if (sequence !== this.leaveSequence || roomId !== this.state.room?.roomId) return
    this.capability = null
    this.nextAction = 1
    this.clearPendingCommands()
    this.update({ room: null, snapshot: null, youSeat: null, admissionBaseline: null,
      leavePending: false, settingsPending: false, failure: null })
  }
  private acceptError(code: V14ErrorCode, sequence: number | null): void {
    const admissionError = sequence !== null && sequence === this.admissionSequence
    if (admissionError && this.capability && this.state.admissionPending) {
      this.acceptAdmissionError(code)
      return
    }
    this.controlFailure = null
    if (admissionError) {
      this.admission = null
      this.admissionSequence = null
      this.update({ admissionPending: false })
    }
    if (sequence !== null && sequence === this.leaveSequence) {
      this.leaveSequence = null; this.update({ leavePending: false })
    }
    if (sequence !== null && sequence === this.settingsSequence) {
      this.settingsSequence = null
      this.pendingSettings = null
      this.update({ settingsPending: false })
    }
    const control = sequence === null ? undefined : this.sentControls.find((item) => item.sequence === sequence)
    if (control) {
      this.sentControls = this.sentControls.filter((item) => item.sequence !== sequence)
      if ((code === 'invalidPhase' || code === 'staleRoom') && controlIsObsolete(control, this.state.room)) return
      if (code === 'invalidPhase' || code === 'staleRoom') this.controlFailure = { ...control, code }
    }
    this.update({ failure: code })
  }
  private sendAdmission(): boolean {
    if (this.state.connection !== 'authenticated') return false
    if (this.capability && !this.recovery.hasTime()) { this.terminal('connectionFailed', true); return false }
    const sequence = this.sequence()
    if (sequence === null) return false
    let message: GameClientMessage
    if (this.capability) {
      message = { type: 'resumeRoom', sequence, roomId: this.capability.roomId, resumeToken: this.capability.token }
    } else if (this.admission?.type === 'create') {
      message = { type: 'createRoom', sequence, configuration: createConfiguration(this.admission) }
    }
    else if (this.admission?.type === 'join') message = { type: 'joinRoom', sequence, roomCode: this.admission.roomCode }
    else return false
    if (!this.send(message)) return false
    this.admissionSequence = sequence
    return true
  }
  private sendRoomCommand(command: RoomCommand): boolean {
    const room = this.state.room
    const sequence = this.sequence()
    if (!room || sequence === null) return false
    if (!this.send({ ...command, sequence, roomId: room.roomId, roundId: room.roundId } as GameClientMessage)) return false
    if (command.type === 'movement') {
      this.sentControls.push({ sequence, roomId: room.roomId, roundId: room.roundId, revision: room.revision })
      if (this.sentControls.length > 64) this.sentControls.shift()
    }
    return true
  }
  private sequence(): number | null {
    if (this.nextSequence > MAX_COUNTER) { this.update({ failure: 'sequenceExhausted' }); return null }
    return this.nextSequence++
  }
  private send(message: GameClientMessage): boolean {
    if (this.state.connection !== 'authenticated' || this.socket?.readyState !== WebSocket.OPEN) return false
    this.socket.send(JSON.stringify(message))
    return true
  }
  private closed(generation: number, socket: WebSocket, code: number, wasResume: boolean): void {
    if (!isCurrentConnection(generation, socket, this.generation, this.socket)) return
    this.recovery.cancelRetry()
    const leaving = this.leaveSequence !== null
    this.socket = null
    this.controller = null
    this.clearPendingCommands()
    this.nextSequence = 1
    if (code === 4001) { this.terminal('seatReplaced'); return }
    if (this.stopped) return
    if (leaving) { this.terminal('connectionFailed'); return }
    if (this.capability) { this.recovery.start(); this.scheduleReconnect(); return }
    this.retryOrFail(wasResume, 'connectionFailed')
  }
  private scheduleReconnect(): void {
    const attempt = this.state.reconnectAttempt + 1
    const generation = this.generation
    if (attempt > MAX_RECONNECT_ATTEMPTS) { this.terminal('connectionFailed'); return }
    this.update({ connection: 'reconnecting', reconnectAttempt: attempt, admissionPending: true,
      settingsPending: false, failure: null })
    if (!this.recovery.schedule(attempt * 250, () => {
      if (generation === this.generation && !this.stopped && this.capability
        && this.state.admissionPending && this.leaveSequence === null) {
        void this.connect(true)
      }
    })) this.terminal('connectionFailed', true)
  }
  private retryThrottledResume(): void {
    const delay = this.recovery.nextRateLimitDelay()
    const generation = this.generation
    const socket = this.socket
    if (delay === null || !socket || !this.recovery.schedule(delay, () => {
      if (isCurrentConnection(generation, socket, this.generation, this.socket)
        && this.state.connection === 'authenticated' && this.state.admissionPending
        && this.admissionSequence === null && this.leaveSequence === null && this.capability) this.sendAdmission()
    })) this.terminal(this.recovery.hasTime() ? 'rateLimited' : 'connectionFailed', true)
  }
  private acceptAdmissionError(code: V14ErrorCode): void {
    this.admissionSequence = null
    if (code === 'rateLimited' && this.recovery.hasTime()) { this.retryThrottledResume(); return }
    this.terminal(code === 'rateLimited' ? 'connectionFailed' : code, true)
  }
  private retryOrFail(resume: boolean, failure: 'connectionFailed' | 'sessionFailed'): void {
    this.socket = null
    this.controller = null
    if (resume && this.capability) this.scheduleReconnect()
    else {
      this.admission = null
      this.admissionSequence = null
      this.update({ connection: 'disconnected', admissionPending: false, failure })
    }
  }
  private terminal(failure: GameFailure, close = false): void {
    this.stopped = true
    this.generation += 1
    this.controller?.abort()
    this.controller = null
    this.recovery.complete()
    this.clearMembership()
    const socket = this.socket
    this.socket = null
    if (close) socket?.close()
    this.update({
      connection: 'disconnected', room: null, snapshot: null, youSeat: null,
      admissionBaseline: null,
      admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0, failure,
    })
  }
  private clearObsoleteControlFailure(room: RoomView): { failure?: null } {
    const failure = this.controlFailure
    if (!failure || this.state.failure !== failure.code || !controlIsObsolete(failure, room)) return {}
    this.controlFailure = null
    return { failure: null }
  }
  private clearPendingCommands(): void {
    this.admissionSequence = null; this.leaveSequence = null; this.settingsSequence = null
    this.pendingSettings = null
    this.sentControls = []
    this.controlFailure = null
  }
  private clearMembership(): void {
    this.capability = null; this.admission = null
    this.clearPendingCommands()
    this.nextSequence = 1; this.nextAction = 1
  }
  private update(change: Partial<GameClientState>): void { this.state = { ...this.state, ...change }; this.onChange(this.state) }
}
