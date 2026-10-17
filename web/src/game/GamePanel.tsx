import { Alert, Button, Group, Stack, Text, TextInput, Title } from '@mantine/core'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'
import { GameArena } from './arena/GameArena'
import type { Direction } from './arena/types'
import type { GameClientState, GameFailure } from './client'
import type { RoomPhase, Seat } from './schema'
import { useGameClient } from './useGameClient'

const CODE_PATTERN = /^[A-HJ-NP-Z2-9]{8}$/

function timerTicks(state: GameClientState): number | null {
  const phase = state.room?.phase
  if (phase?.type === 'countdown') return Math.max(0, phase.endsAt - (state.room?.tick ?? 0))
  if (phase?.type === 'running' || phase?.type === 'result') return state.snapshot?.state.remainingTicks ?? null
  return null
}

function clock(ticks: number | null): string {
  if (ticks === null) return '--:--'
  const seconds = Math.ceil(ticks / 60)
  return `${Math.floor(seconds / 60)}:${String(seconds % 60).padStart(2, '0')}`
}

function ownActorAlive(state: GameClientState): boolean {
  return Boolean(
    state.youSeat && state.snapshot?.state.actors.find((actor) => actor.seat === state.youSeat)?.alive,
  )
}

function phaseKey(phase: RoomPhase): `game.phase.${RoomPhase['type']}` {
  return `game.phase.${phase.type}`
}

function errorKey(failure: GameFailure): `game.error.${GameFailure}` {
  return `game.error.${failure}`
}

function RoomOutcome({ phase }: { phase: RoomPhase }) {
  const { t } = useTranslation()
  if (phase.type === 'result') {
    if (phase.result.type === 'winner') return <Text role="status">{t('game.result.winner', { player: t(`game.player.${phase.result.seat}`) })}</Text>
    return <Text role="status">{t(`game.result.${phase.result.reason}`)}</Text>
  }
  if (phase.type === 'interrupted') return <Text role="status">{t(`game.interruption.${phase.reason}`)}</Text>
  return null
}

function SeatLine({ state, seat }: { state: GameClientState; seat: Seat }) {
  const { t } = useTranslation()
  const room = state.room!
  const status = room.seats.find((candidate) => candidate.seat === seat)
  const actor = room.phase.type === 'waiting' || room.phase.type === 'countdown'
    ? undefined
    : state.snapshot?.state.actors.find((candidate) => candidate.seat === seat)
  const seatState = actor
    ? (actor.alive ? 'alive' : 'eliminated')
    : status?.occupied ? (status.ready ? 'ready' : 'notReady') : 'open'
  return (
    <li className={`seat-line seat-${seat}`}>
      <span className="player-mark" aria-hidden="true" />
      <div className="seat-details">
        <span>
          <strong>{t(`game.player.${seat}`)}</strong>{state.youSeat === seat ? ` ${t('game.you')}` : ''}
          <small>{t(`game.seat.${seatState}`)}</small>
        </span>
        {actor && status?.occupied && (
          <dl className="actor-stats" aria-label={t('game.stats.label', { player: t(`game.player.${seat}`) })}>
            <div><dt>{t('game.stats.available')}</dt><dd>{actor.bombCapacity - actor.liveBombCount}</dd></div>
            <div><dt>{t('game.stats.capacity')}</dt><dd>{actor.bombCapacity}</dd></div>
            <div><dt>{t('game.stats.range')}</dt><dd>{actor.bombRange}</dd></div>
            <div><dt>{t('game.stats.speed')}</dt><dd>{actor.speedLevel + 1}</dd></div>
          </dl>
        )}
      </div>
    </li>
  )
}

function LobbyControls({ state, onCreate, onJoin }: {
  state: GameClientState
  onCreate: () => void
  onJoin: (code: string) => void
}) {
  const { t } = useTranslation()
  const [code, setCode] = useState('')
  const busy = ['connecting', 'authenticating'].includes(state.connection) || state.admissionPending
  return (
    <div className="lobby-controls">
      <Title order={2}>{t('game.lobby.title')}</Title>
      <Text>{t('game.lobby.explanation')}</Text>
      <Button size="lg" onClick={onCreate} loading={busy}>{t('game.lobby.create')}</Button>
      <div className="join-rule"><span>{t('game.lobby.or')}</span></div>
      <TextInput
        label={t('game.lobby.code')}
        value={code}
        maxLength={8}
        autoComplete="off"
        spellCheck={false}
        onChange={(event) => setCode(event.currentTarget.value.toUpperCase())}
        error={code.length > 0 && !CODE_PATTERN.test(code) ? t('game.lobby.codeHint') : undefined}
      />
      <Button variant="outline" disabled={busy || !CODE_PATTERN.test(code)} onClick={() => onJoin(code)}>
        {t('game.lobby.join')}
      </Button>
    </div>
  )
}

function RoomControls({ state, onReady, onLeave }: {
  state: GameClientState
  onReady: (ready: boolean) => void
  onLeave: () => void
}) {
  const { t } = useTranslation()
  const room = state.room
  if (!room) return null
  const ownStatus = room.seats.find((seat) => seat.seat === state.youSeat)
  const canReady = !state.leavePending && (room.phase.type === 'waiting' || room.phase.type === 'countdown')
  return (
    <div className="room-controls">
      <div>
        <Text size="sm">{t('game.room.invitation')}</Text>
        <Text className="room-code" aria-label={t('game.room.codeLabel')}>{room.roomCode}</Text>
      </div>
      <Text className="phase-line">{t(phaseKey(room.phase))}</Text>
      <ol className="seat-list" aria-label={t('game.room.players')}>
        <SeatLine state={state} seat="one" />
        <SeatLine state={state} seat="two" />
      </ol>
      <RoomOutcome phase={room.phase} />
      <Stack gap="sm">
        {canReady && ownStatus && (
          <Button onClick={() => onReady(!ownStatus.ready)}>
            {t(ownStatus.ready ? 'game.room.cancelReady' : 'game.room.ready')}
          </Button>
        )}
        <Button variant="subtle" color="dark" loading={state.leavePending} onClick={onLeave}>
          {t('game.room.leave')}
        </Button>
      </Stack>
    </div>
  )
}

export function GamePanel() {
  const { t } = useTranslation()
  const [state, client] = useGameClient()
  const failureText = state.failure ? t(errorKey(state.failure)) : ''
  const enabled = state.connection === 'authenticated'
    && !state.leavePending
    && state.room?.phase.type === 'running'
    && ownActorAlive(state)
  const waitingText = state.room ? t(phaseKey(state.room.phase)) : t(`game.connection.${state.connection}`)

  function direction(value: Direction | null): void { client.movement(value) }

  return (
    <section className="game-shell" aria-label={t('game.label')}>
      <div className="arena-column">
        <div className="match-strip">
          <span>{t('game.player.one')}</span>
          <time aria-label={t('game.timer')}>{clock(timerTicks(state))}</time>
          <span>{t('game.player.two')}</span>
        </div>
        <GameArena
          label={t('game.arena.label')}
          waitingText={waitingText}
          failureText={t('game.arena.failure')}
          focusHint={t('game.arena.focusHint')}
          instructionsId="game-instructions"
          snapshot={state.snapshot}
          youSeat={state.youSeat}
          enabled={enabled}
          onDirection={direction}
          onBomb={() => client.placeBomb()}
        />
        <p id="game-instructions" className="game-instructions">{t('game.arena.instructions')}</p>
      </div>
      <aside className="control-column" aria-live="polite">
        {failureText && <Alert color="red" title={t('game.error.title')}>{failureText}</Alert>}
        {state.room ? (
          <RoomControls state={state} onReady={(ready) => client.ready(ready)} onLeave={() => client.leave()} />
        ) : (
          <LobbyControls
            state={state}
            onCreate={() => client.admit({ type: 'create' })}
            onJoin={(roomCode) => client.admit({ type: 'join', roomCode })}
          />
        )}
        {state.connection !== 'disconnected' && !state.room && (
          <Group gap="xs" className="connection-note">
            <span className="connection-dot" aria-hidden="true" />
            <Text size="sm">{t(`game.connection.${state.connection}`)}</Text>
          </Group>
        )}
      </aside>
    </section>
  )
}
