import { Alert, Button, Group, NativeSelect, Stack, Text, TextInput, Title } from '@mantine/core'
import { useMemo, useState } from 'react'
import type { ReactNode } from 'react'
import type { KeyboardBindings } from '../arena/bindings'
import { GameArena } from '../arena/GameArena'
import type { Direction } from '../arena/types'
import type { AdmissionIntent, GameClientState } from './client'
import { format, useGameCopy, type GameCopy } from './locale'
import type {
  BoardSize, Difficulty, GameSettings, MatchInterruptReason, MatchResult, RoomConfiguration, RoomPhase, RoundResult,
  Seat, VisibleActorView,
} from '../v17/schema'
import { useGameClient } from './useGameClient'
import { ControlsDialog } from './ControlsDialog'
import { bindingsSummary, keyLabel } from './controlLabels'
import { loadControlBindings, saveControlBindings } from './controlPreferences'
import { SoundControl } from '../audio/SoundControl'
import { GameplaySettingsPanel } from './GameplaySettingsPanel'
import { presentSnapshot } from './presentation'
import { ActorDetails, OwnStatusBar } from './OwnStatusBar'
import { OwnPickupNotice } from './OwnPickupNotice'
import { controllersAvailable, isLegacySolo, roomCopy } from './roomControllers'
import { CpuDebugPanel } from './CpuDebugPanel'
import { useCpuDebug } from './useCpuDebug'
import { cpuDebugOptedIn } from './cpuDebugOptIn'

const CODE_PATTERN = /^[A-HJ-NP-Z2-9]{8}$/
const BOARD_PRESETS = [
  { width: 15, height: 13 }, { width: 13, height: 11 }, { width: 11, height: 9 },
] as const satisfies readonly BoardSize[]
const boardKey = ({ width, height }: BoardSize): string => `${width}x${height}`

export interface GameActions {
  admit(intent: AdmissionIntent): boolean
  ready(ready: boolean): boolean
  updateSettings(settings: GameSettings): boolean
  rematch(ready: boolean): boolean
  movement(direction: Direction | null): boolean
  placeBomb(): boolean
  useEquipment(): boolean
  leave(): boolean
}

function timerTicks(state: GameClientState): number | null {
  const room = state.room
  if (!room) return null
  const phase = room.phase
  if (phase.type === 'countdown') return Math.max(0, phase.endsAt - room.tick)
  if (phase.type === 'result') return Math.max(0, phase.holdUntil - room.tick)
  if (phase.type === 'matchResult') return Math.max(0, phase.rematchAt - room.tick)
  if (phase.type === 'running') return state.snapshot?.state.remainingTicks ?? null
  return null
}

function clock(ticks: number | null): string {
  if (ticks === null) return '--:--'
  const seconds = Math.ceil(ticks / 60)
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
}

function phaseText(copy: GameCopy, phase: RoomPhase): string {
  if (phase.type === 'countdown') {
    return phase.kind === 'initial' ? copy.phase.initialCountdown : copy.phase.interRoundCountdown
  }
  return copy.phase[phase.type]
}

function reasonText(copy: GameCopy, reason: MatchInterruptReason): string {
  return copy.interruption[reason]
}

function RoundOutcome({ copy, result }: { copy: GameCopy; result: RoundResult }) {
  if (result.type === 'winner') {
    return <Text role="status">{format(copy.roundResult.winner, { player: copy.player[result.seat] })}</Text>
  }
  return <Text role="status">{copy.roundResult[result.reason]}</Text>
}

function MatchOutcome({ copy, result, wins }: { copy: GameCopy; result: MatchResult; wins: number }) {
  if (result.type === 'winner') {
    const template = result.reason === 'roundWins'
      ? copy.matchResult[wins === 1 ? 'roundWinConfigured' : 'roundWinsConfigured']
      : copy.matchResult.forfeit
    return <Text role="status">{format(template, { player: copy.player[result.seat], wins })}</Text>
  }
  if (result.type === 'abandoned') return <Text role="status">{copy.matchResult.abandoned}</Text>
  return <Text role="status">{format(copy.matchResult.interrupted, { reason: reasonText(copy, result.reason) })}</Text>
}

function AuthoritativeOutcome({ state, copy }: { state: GameClientState; copy: GameCopy }) {
  const room = state.room
  if (!room) return null
  if (room.phase.type === 'result' && room.lastRound) return <RoundOutcome copy={copy} result={room.lastRound.result} />
  if (room.phase.type === 'matchResult') return <MatchOutcome copy={copy} result={room.phase.result} wins={room.settings.winsToFinish} />
  if (room.phase.type === 'interrupted') return <Text role="status">{reasonText(copy, room.phase.reason)}</Text>
  return null
}

function seatDescription(state: GameClientState, seat: Seat): keyof GameCopy['seat'] {
  const room = state.room!
  const status = room.seats.find((candidate) => candidate.seat === seat)
  const actor = state.snapshot?.state.actors.find((candidate) => candidate.seat === seat)
  if (status?.connection.type === 'disconnected') return 'disconnected'
  if (status?.connection.type === 'expired') return 'expired'
  else if (status?.occupied && room.phase.type === 'matchResult') {
    return status.rematchReady ? 'rematchReady' : 'rematchNotReady'
  } else if (status?.occupied && actor && ['running', 'result'].includes(room.phase.type)) {
    return actor.alive ? 'alive' : 'eliminated'
  } else if (status?.occupied) return status.ready ? 'ready' : 'notReady'
  return 'open'
}

function visibleActor(state: GameClientState, seat: Seat): VisibleActorView | null {
  const actor = state.snapshot?.state.actors.find((candidate) => candidate.seat === seat)
  return actor?.type === 'visible' ? actor : null
}

function SeatLine({ state, seat, copy }: { state: GameClientState; seat: Seat; copy: GameCopy }) {
  const room = state.room!
  const status = room.seats.find((candidate) => candidate.seat === seat)
  const actor = visibleActor(state, seat)
  const seatState = seatDescription(state, seat)
  const grace = status?.connection.type === 'disconnected'
    ? clock(Math.max(0, status.connection.expiresAt - room.tick)) : null
  return (
    <li className={`seat-line seat-${seat}`}>
      <span className="player-mark" aria-hidden="true" />
      <div className="seat-details">
        <span>
          <strong>{copy.player[seat]}</strong>{state.youSeat === seat ? ` ${copy.you}` : ''}
          <small>{copy.seat[seatState]}</small>
          {grace && <small role="timer">{format(copy.reconnect.grace, { time: grace })}</small>}
        </span>
        {actor && status?.occupied && state.youSeat !== seat && (
          <dl className="actor-stats" aria-label={format(copy.stats.label, { player: copy.player[seat] })}>
            <ActorDetails actor={actor} copy={copy} />
          </dl>
        )}
      </div>
    </li>
  )
}

function LobbyControls({ state, actions, copy }: { state: GameClientState; actions: GameActions; copy: GameCopy }) {
  const [code, setCode] = useState('')
  const [difficulty, setDifficulty] = useState<Difficulty>('standard')
  const [playerCount, setPlayerCount] = useState(2)
  const [board, setBoard] = useState<BoardSize>(BOARD_PRESETS[0])
  const [controllers, setControllers] = useState<Array<'human' | 'cpu'>>(['human', 'human'])
  const busy = state.admissionPending
    || ['connecting', 'authenticating', 'reconnecting'].includes(state.connection)
  const selectDifficulty = (value: string): void => {
    if (value === 'easy' || value === 'standard') setDifficulty(value)
  }
  const resizeControllers = (count: number): void => {
    if (count < 2 || count > 4) return
    setPlayerCount(count)
    setControllers((current) => Array.from({ length: count }, (_, index) => current[index] ?? 'human'))
  }
  const setController = (index: number, value: string): void => {
    if (index === 0 || (value !== 'human' && value !== 'cpu')) return
    setControllers((current) => current.map((controller, position) => position === index ? value : controller))
  }
  const selectBoard = (value: string): void => {
    const selected = BOARD_PRESETS.find((preset) => boardKey(preset) === value)
    if (selected) setBoard(selected)
  }
  const customConfiguration = (): RoomConfiguration => {
    const hasCpu = controllers.includes('cpu')
    return {
      controllers, cpuDifficulty: hasCpu ? difficulty : null, board,
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 } },
    } as RoomConfiguration
  }
  return (
    <div className="lobby-controls">
      <Title order={2}>{copy.lobby.title}</Title>
      <Text>{copy.lobby.explanation}</Text>
      <Button size="lg" onClick={() => actions.admit({ type: 'create' })} loading={busy}>{copy.lobby.create}</Button>
      <NativeSelect label={copy.solo.difficultyLabel} value={difficulty} disabled={busy}
        onChange={(event) => selectDifficulty(event.currentTarget.value)} data={[
          { value: 'easy', label: copy.solo.easy },
          { value: 'standard', label: copy.solo.standard },
        ]} />
      <Button size="lg" variant="light" disabled={busy}
        onClick={() => actions.admit({ type: 'create', mode: { type: 'solo', difficulty } })}>
        {copy.solo.play}
      </Button>
      <fieldset className="custom-match">
        <legend>{copy.multiplayer.title}</legend>
        <NativeSelect label={copy.multiplayer.playerCount} value={String(playerCount)} disabled={busy}
          onChange={(event) => resizeControllers(Number(event.currentTarget.value))}
          data={['2', '3', '4']} />
        <NativeSelect label={copy.multiplayer.boardSize} value={boardKey(board)} disabled={busy}
          onChange={(event) => selectBoard(event.currentTarget.value)}
          data={BOARD_PRESETS.map((preset) => ({
            value: boardKey(preset),
            label: format(copy.multiplayer.boardDimensions, preset),
          }))} />
        {controllers.slice(1).map((controller, index) => <NativeSelect
          key={index + 1} label={format(copy.multiplayer.controller, { player: index + 2 })}
          value={controller} disabled={busy} onChange={(event) => setController(index + 1, event.currentTarget.value)}
          data={[{ value: 'human', label: copy.multiplayer.human }, { value: 'cpu', label: copy.multiplayer.cpu }]} />)}
        {controllers.includes('cpu') && <NativeSelect label={copy.solo.difficultyLabel} value={difficulty}
          disabled={busy} onChange={(event) => selectDifficulty(event.currentTarget.value)} data={[
            { value: 'easy', label: copy.solo.easy }, { value: 'standard', label: copy.solo.standard },
          ]} />}
        <Button size="lg" variant="light" disabled={busy}
          onClick={() => actions.admit({ type: 'create', configuration: customConfiguration() })}>
          {copy.multiplayer.create}
        </Button>
      </fieldset>
      <div className="join-rule"><span>{copy.lobby.or}</span></div>
      <TextInput
        label={copy.lobby.code} value={code} maxLength={8} autoComplete="off" spellCheck={false}
        onChange={(event) => setCode(event.currentTarget.value.toUpperCase())}
        error={code.length > 0 && !CODE_PATTERN.test(code) ? copy.lobby.codeHint : undefined}
      />
      <Button variant="outline" disabled={busy || !CODE_PATTERN.test(code)}
        onClick={() => actions.admit({ type: 'join', roomCode: code })}>{copy.lobby.join}</Button>
    </div>
  )
}

function RematchControl({ state, actions, copy }: { state: GameClientState; actions: GameActions; copy: GameCopy }) {
  const room = state.room!
  if (room.phase.type !== 'matchResult') return null
  const own = room.seats.find((seat) => seat.seat === state.youSeat)
  if (!controllersAvailable(room)) return <Text>{copy.rematch.needsPlayers}</Text>
  if (room.tick < room.phase.rematchAt) return <Text>{copy.rematch.locked}</Text>
  if (!own) return null
  return (
    <Button disabled={state.connection !== 'authenticated' || state.leavePending || state.admissionPending}
      onClick={() => actions.rematch(!own.rematchReady)}>
      {own.rematchReady ? copy.rematch.withdraw : copy.rematch.ready}
    </Button>
  )
}

function RoomControls({ state, actions, copy }: { state: GameClientState; actions: GameActions; copy: GameCopy }) {
  const room = state.room!
  const boardDimensions = {
    width: room.configuration.board.width, height: room.configuration.board.height,
  }
  const own = room.seats.find((seat) => seat.seat === state.youSeat)
  const connected = own?.connection.type === 'connected' && state.connection === 'authenticated'
  const canReady = room.phase.type === 'waiting'
    || (room.phase.type === 'countdown' && room.phase.kind === 'initial')
  const held = room.phase.type === 'result' && room.tick >= room.phase.holdUntil
    && !controllersAvailable(room)
  const humanVacancy = room.seats.some(({ controller, occupied }) => controller.type === 'human' && !occupied)
  return (
    <div className="room-controls">
      <Text>{format(copy.room.boardSize, boardDimensions)}</Text>
      {room.configuration.cpuDifficulty && <Text>{isLegacySolo(room)
        ? copy.solo.roomDifficulty[room.configuration.cpuDifficulty]
        : format(copy.multiplayer.cpuDifficulty, {
          difficulty: copy.solo[room.configuration.cpuDifficulty],
        })}</Text>}
      {(!room.configuration.cpuDifficulty || humanVacancy) &&
        <div><Text size="sm">{copy.room.invitation}</Text>
          <Text className="room-code" aria-label={copy.room.codeLabel}>{room.roomCode}</Text></div>}
      <Text className="phase-line" aria-live="polite">{phaseText(copy, room.phase)}</Text>
      {held && <Text role="status">{copy.reconnect.resultHeld}</Text>}
      <ol className="seat-list" aria-label={copy.room.players}>
        {room.seats.map(({ seat }) => <SeatLine key={seat} state={state} seat={seat} copy={copy} />)}
      </ol>
      <AuthoritativeOutcome state={state} copy={copy} />
      <GameplaySettingsPanel key={room.settingsRevision} room={room} youSeat={state.youSeat}
        pending={state.settingsPending || state.admissionPending} copy={copy}
        onApply={(settings) => actions.updateSettings(settings)} />
      <Stack gap="sm">
        {canReady && own && <Button disabled={!connected || state.leavePending || state.admissionPending
          || state.settingsPending}
          onClick={() => actions.ready(!own.ready)}>{own.ready ? copy.room.cancelReady : copy.room.ready}</Button>}
        <RematchControl state={state} actions={actions} copy={copy} />
        <Button variant="subtle" color="dark"
          disabled={state.connection !== 'authenticated' || state.admissionPending}
          loading={state.leavePending} onClick={() => actions.leave()}>{copy.room.leave}</Button>
      </Stack>
    </div>
  )
}

function ownActorAlive(state: GameClientState): boolean {
  return Boolean(state.youSeat && visibleActor(state, state.youSeat)?.alive)
}

function gameInputEnabled(state: GameClientState): boolean {
  return state.connection === 'authenticated' && !state.admissionPending && !state.leavePending
    && state.room?.phase.type === 'running' && ownActorAlive(state)
}

function ownStatusActor(state: GameClientState): VisibleActorView | null {
  if (state.connection !== 'authenticated' || state.admissionPending || !state.youSeat) return null
  return visibleActor(state, state.youSeat)
}

function arenaFocusHint(state: GameClientState, snapshot: ReturnType<typeof presentSnapshot>,
  bindings: KeyboardBindings, copy: ReturnType<typeof useGameCopy>): string {
  const actor = state.youSeat && snapshot?.state.actors.find(({ seat }) => seat === state.youSeat)
  const bomb = bindings.bomb.map((code) => keyLabel(code, copy)).join(' / ')
  const equipment = bindings.equipment.map((code) => keyLabel(code, copy)).join(' / ')
  if (actor?.invisibleBombsRemainingTicks && state.room?.settings.enabledItems.includes('remoteBomb')) {
    return format(copy.arena.blindBombHint, { bomb })
  }
  if (actor?.remoteArmed) return format(copy.arena.remoteHint, { bomb })
  return actor?.punchEquipped
    ? format(copy.arena.punchHint, { equipment }) : copy.arena.focusHint
}

export function GamePanelView({ state, actions, developerPanel }: {
  state: GameClientState
  actions: GameActions
  developerPanel?: ReactNode
}) {
  const translatedCopy = useGameCopy()
  const copy = useMemo(() => roomCopy(translatedCopy, state.room), [translatedCopy, state.room])
  const [bindings, setBindings] = useState(loadControlBindings)
  const [controlsOpen, setControlsOpen] = useState(false)
  const failureText = state.failure ? copy.error[state.failure as keyof typeof copy.error] : ''
  const enabled = !controlsOpen && gameInputEnabled(state)
  const waitingText = state.room ? phaseText(copy, state.room.phase) : copy.connection[state.connection]
  const round = state.room?.roundNumber
  const reconnecting = state.room && (state.admissionPending
    || ['reconnecting', 'connecting', 'authenticating'].includes(state.connection))
  const arenaSnapshot = useMemo(() => presentSnapshot(state), [state])
  const ownActor = ownStatusActor(state)
  const focusHint = arenaFocusHint(state, arenaSnapshot, bindings, copy)
  const updateBindings = (next: KeyboardBindings): void => {
    setBindings(next)
    saveControlBindings(next)
  }
  return (
    <section className="game-shell" aria-label={copy.label}>
      <div className="arena-column">
        <div className="match-strip">
          {state.room?.scores.slice(0, Math.ceil(state.room.scores.length / 2)).map(({ seat, wins }) =>
            <span key={seat} className={`match-score seat-${seat}`} role="group"
              aria-label={format(copy.scoreLabel, { player: copy.player[seat] })}>
              <span>{copy.player[seat]}</span><strong>{wins}</strong></span>)}
          <span className="match-clock"><time aria-label={copy.timer}>{clock(timerTicks(state))}</time>
            <small>{round ? format(copy.room.round, { round }) : copy.room.beforeFirstRound}</small></span>
          {state.room?.scores.slice(Math.ceil(state.room.scores.length / 2)).map(({ seat, wins }) =>
            <span key={seat} className={`match-score seat-${seat}`} role="group"
              aria-label={format(copy.scoreLabel, { player: copy.player[seat] })}>
              <span>{copy.player[seat]}</span><strong>{wins}</strong></span>)}
        </div>
        <GameArena label={copy.arena.label} waitingText={waitingText} failureText={copy.arena.failure}
          focusHint={focusHint} instructionsId="game-instructions" snapshot={arenaSnapshot}
          youSeat={state.youSeat} enabled={Boolean(enabled)}
          bindings={bindings}
          onDirection={(direction) => actions.movement(direction)} onBomb={() => actions.placeBomb()}
          onEquipment={() => actions.useEquipment()} />
        {ownActor && <OwnStatusBar actor={ownActor} copy={copy} />}
        <OwnPickupNotice state={state} copy={copy} />
        <p id="game-instructions" className="game-instructions">{bindingsSummary(bindings, copy)}</p>
        <div className="local-controls">
          <Button className="controls-button" variant="subtle" color="dark" size="compact-sm"
            onClick={() => setControlsOpen(true)}>{copy.controls.open}</Button>
          <SoundControl state={state} copy={copy} />
        </div>
      </div>
      <aside className="control-column">
        {reconnecting && <Alert color="yellow" title={copy.reconnect.title}>{copy.reconnect.retrying}</Alert>}
        {failureText && <Alert color="red" title={copy.error.title}>{failureText}</Alert>}
        {state.room ? <RoomControls state={state} actions={actions} copy={copy} />
          : <LobbyControls state={state} actions={actions} copy={copy} />}
        {developerPanel}
        {state.connection !== 'disconnected' && !state.room && <Group gap="xs" className="connection-note">
          <span className="connection-dot" aria-hidden="true" /><Text size="sm">{copy.connection[state.connection]}</Text>
        </Group>}
      </aside>
      <ControlsDialog opened={controlsOpen} bindings={bindings} copy={copy}
        onChange={updateBindings} onClose={() => setControlsOpen(false)} />
    </section>
  )
}

export function GamePanel() {
  const [state, client] = useGameClient()
  const debugEnabled = cpuDebugOptedIn(window.location.search)
  const debug = useCpuDebug(state, client, debugEnabled)
  return <GamePanelView state={state} actions={client}
    developerPanel={debugEnabled && state.room ? <CpuDebugPanel state={state} debug={debug} /> : null} />
}
