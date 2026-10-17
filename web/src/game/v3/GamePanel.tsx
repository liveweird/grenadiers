import { Alert, Button, Group, Stack, Text, TextInput, Title } from '@mantine/core'
import { useState } from 'react'
import { GameArena } from '../arena/GameArena'
import type { Direction } from '../arena/types'
import type { AdmissionIntent, GameClientState, GameFailure } from './client'
import { format, useV3Copy, type V3Copy } from './locale'
import type { MatchInterruptReason, MatchResult, RoomPhase, RoundResult, Seat } from './schema'
import { useGameClient } from './useGameClient'

const CODE_PATTERN = /^[A-HJ-NP-Z2-9]{8}$/

export interface GameActions {
  admit(intent: AdmissionIntent): boolean
  ready(ready: boolean): boolean
  rematch(ready: boolean): boolean
  movement(direction: Direction | null): boolean
  placeBomb(): boolean
  leave(): boolean
}

function timerTicks(state: GameClientState): number | null {
  const room = state.room
  if (!room) return null
  const phase = room.phase
  if (phase.type === 'countdown') return Math.max(0, phase.endsAt - room.tick)
  if (phase.type === 'result') return Math.max(0, phase.nextCountdownAt - room.tick)
  if (phase.type === 'matchResult') return Math.max(0, phase.rematchAt - room.tick)
  if (phase.type === 'running') return state.snapshot?.state.remainingTicks ?? null
  return null
}

function clock(ticks: number | null): string {
  if (ticks === null) return '--:--'
  const seconds = Math.ceil(ticks / 60)
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
}

function interpolate(value: string, values: Record<string, string | number>): string {
  return format(value, values)
}

function phaseText(copy: V3Copy, phase: RoomPhase): string {
  if (phase.type === 'countdown') {
    return phase.kind === 'initial' ? copy.phase.initialCountdown : copy.phase.interRoundCountdown
  }
  return copy.phase[phase.type]
}

function reasonText(copy: V3Copy, reason: MatchInterruptReason): string {
  return copy.interruption[reason]
}

function RoundOutcome({ copy, result }: { copy: V3Copy; result: RoundResult }) {
  if (result.type === 'winner') {
    return <Text role="status">{interpolate(copy.roundResult.winner, { player: copy.player[result.seat] })}</Text>
  }
  return <Text role="status">{copy.roundResult[result.reason]}</Text>
}

function MatchOutcome({ copy, result }: { copy: V3Copy; result: MatchResult }) {
  if (result.type === 'winner') {
    return <Text role="status">{interpolate(copy.matchResult[result.reason], { player: copy.player[result.seat] })}</Text>
  }
  if (result.type === 'abandoned') return <Text role="status">{copy.matchResult.abandoned}</Text>
  return <Text role="status">{interpolate(copy.matchResult.interrupted, { reason: reasonText(copy, result.reason) })}</Text>
}

function AuthoritativeOutcome({ state, copy }: { state: GameClientState; copy: V3Copy }) {
  const room = state.room
  if (!room) return null
  if (room.phase.type === 'result' && room.lastRound) return <RoundOutcome copy={copy} result={room.lastRound.result} />
  if (room.phase.type === 'matchResult') return <MatchOutcome copy={copy} result={room.phase.result} />
  if (room.phase.type === 'interrupted') {
    return <Text role="status">{reasonText(copy, room.phase.reason)}</Text>
  }
  return null
}

function SeatLine({ state, seat, copy }: { state: GameClientState; seat: Seat; copy: V3Copy }) {
  const room = state.room!
  const status = room.seats.find((candidate) => candidate.seat === seat)
  const actor = state.snapshot?.state.actors.find((candidate) => candidate.seat === seat)
  let seatState: keyof V3Copy['seat'] = 'open'
  if (status?.occupied && room.phase.type === 'matchResult') {
    seatState = status.rematchReady ? 'rematchReady' : 'rematchNotReady'
  } else if (status?.occupied && actor && ['running', 'result'].includes(room.phase.type)) {
    seatState = actor.alive ? 'alive' : 'eliminated'
  } else if (status?.occupied) {
    seatState = status.ready ? 'ready' : 'notReady'
  }
  return (
    <li className={`seat-line seat-${seat}`}>
      <span className="player-mark" aria-hidden="true" />
      <div className="seat-details">
        <span>
          <strong>{copy.player[seat]}</strong>{state.youSeat === seat ? ` ${copy.you}` : ''}
          <small>{copy.seat[seatState]}</small>
        </span>
        {actor && status?.occupied && (
          <dl className="actor-stats" aria-label={interpolate(copy.stats.label, { player: copy.player[seat] })}>
            <div><dt>{copy.stats.available}</dt><dd>{actor.bombCapacity - actor.liveBombCount}</dd></div>
            <div><dt>{copy.stats.capacity}</dt><dd>{actor.bombCapacity}</dd></div>
            <div><dt>{copy.stats.range}</dt><dd>{actor.bombRange}</dd></div>
            <div><dt>{copy.stats.speed}</dt><dd>{actor.speedLevel + 1}</dd></div>
          </dl>
        )}
      </div>
    </li>
  )
}

function LobbyControls({ state, actions, copy }: {
  state: GameClientState; actions: GameActions; copy: V3Copy
}) {
  const [code, setCode] = useState('')
  const busy = ['connecting', 'authenticating'].includes(state.connection) || state.admissionPending
  return (
    <div className="lobby-controls">
      <Title order={2}>{copy.lobby.title}</Title>
      <Text>{copy.lobby.explanation}</Text>
      <Button size="lg" onClick={() => actions.admit({ type: 'create' })} loading={busy}>{copy.lobby.create}</Button>
      <div className="join-rule"><span>{copy.lobby.or}</span></div>
      <TextInput
        label={copy.lobby.code} value={code} maxLength={8} autoComplete="off" spellCheck={false}
        onChange={(event) => setCode(event.currentTarget.value.toUpperCase())}
        error={code.length > 0 && !CODE_PATTERN.test(code) ? copy.lobby.codeHint : undefined}
      />
      <Button variant="outline" disabled={busy || !CODE_PATTERN.test(code)} onClick={() => actions.admit({ type: 'join', roomCode: code })}>
        {copy.lobby.join}
      </Button>
    </div>
  )
}

function RematchControl({ state, actions, copy }: {
  state: GameClientState; actions: GameActions; copy: V3Copy
}) {
  const room = state.room!
  if (room.phase.type !== 'matchResult') return null
  const own = room.seats.find((seat) => seat.seat === state.youSeat)
  const occupied = room.seats.every((seat) => seat.occupied)
  if (!occupied) return <Text>{copy.rematch.needsPlayers}</Text>
  if (room.tick < room.phase.rematchAt) return <Text>{copy.rematch.locked}</Text>
  if (!own) return null
  return (
    <Button disabled={state.leavePending} onClick={() => actions.rematch(!own.rematchReady)}>
      {own.rematchReady ? copy.rematch.withdraw : copy.rematch.ready}
    </Button>
  )
}

function RoomControls({ state, actions, copy }: {
  state: GameClientState; actions: GameActions; copy: V3Copy
}) {
  const room = state.room!
  const own = room.seats.find((seat) => seat.seat === state.youSeat)
  const canReady = room.phase.type === 'waiting'
    || (room.phase.type === 'countdown' && room.phase.kind === 'initial')
  return (
    <div className="room-controls">
      <div>
        <Text size="sm">{copy.room.invitation}</Text>
        <Text className="room-code" aria-label={copy.room.codeLabel}>{room.roomCode}</Text>
      </div>
      <Text className="phase-line">{phaseText(copy, room.phase)}</Text>
      <ol className="seat-list" aria-label={copy.room.players}>
        <SeatLine state={state} seat="one" copy={copy} />
        <SeatLine state={state} seat="two" copy={copy} />
      </ol>
      <AuthoritativeOutcome state={state} copy={copy} />
      <Stack gap="sm">
        {canReady && own && (
          <Button disabled={state.leavePending} onClick={() => actions.ready(!own.ready)}>
            {own.ready ? copy.room.cancelReady : copy.room.ready}
          </Button>
        )}
        <RematchControl state={state} actions={actions} copy={copy} />
        <Button variant="subtle" color="dark" loading={state.leavePending} onClick={() => actions.leave()}>
          {copy.room.leave}
        </Button>
      </Stack>
    </div>
  )
}

function ownActorAlive(state: GameClientState): boolean {
  return Boolean(state.youSeat && state.snapshot?.state.actors.find((actor) => actor.seat === state.youSeat)?.alive)
}

export function GamePanelView({ state, actions }: { state: GameClientState; actions: GameActions }) {
  const copy = useV3Copy()
  const failureText = state.failure ? copy.error[state.failure as GameFailure] : ''
  const enabled = state.connection === 'authenticated' && !state.leavePending
    && state.room?.phase.type === 'running' && ownActorAlive(state)
  const waitingText = state.room ? phaseText(copy, state.room.phase) : copy.connection[state.connection]
  const round = state.room?.roundNumber
  return (
    <section className="game-shell" aria-label={copy.label}>
      <div className="arena-column">
        <div className="match-strip">
          <span className="match-score" role="group" aria-label={interpolate(copy.scoreLabel, { player: copy.player.one })}>
            <span>{copy.player.one}</span><strong>{state.room?.scores.one ?? 0}</strong>
          </span>
          <span className="match-clock">
            <time aria-label={copy.timer}>{clock(timerTicks(state))}</time>
            <small>{round ? interpolate(copy.room.round, { round }) : copy.room.beforeFirstRound}</small>
          </span>
          <span className="match-score" role="group" aria-label={interpolate(copy.scoreLabel, { player: copy.player.two })}>
            <span>{copy.player.two}</span><strong>{state.room?.scores.two ?? 0}</strong>
          </span>
        </div>
        <GameArena
          label={copy.arena.label} waitingText={waitingText} failureText={copy.arena.failure}
          focusHint={copy.arena.focusHint} instructionsId="v3-game-instructions"
          snapshot={state.snapshot} youSeat={state.youSeat} enabled={enabled}
          onDirection={(direction) => actions.movement(direction)} onBomb={() => actions.placeBomb()}
        />
        <p id="v3-game-instructions" className="game-instructions">{copy.arena.instructions}</p>
      </div>
      <aside className="control-column" aria-live="polite">
        {failureText && <Alert color="red" title={copy.error.title}>{failureText}</Alert>}
        {state.room
          ? <RoomControls state={state} actions={actions} copy={copy} />
          : <LobbyControls state={state} actions={actions} copy={copy} />}
        {state.connection !== 'disconnected' && !state.room && (
          <Group gap="xs" className="connection-note">
            <span className="connection-dot" aria-hidden="true" />
            <Text size="sm">{copy.connection[state.connection]}</Text>
          </Group>
        )}
      </aside>
    </section>
  )
}

export function GamePanel() {
  const [state, client] = useGameClient()
  return <GamePanelView state={state} actions={client} />
}
