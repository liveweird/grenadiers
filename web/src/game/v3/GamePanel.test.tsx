import { MantineProvider } from '@mantine/core'
import { cleanup, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameArenaProps } from '../arena/types'
import i18n from '../../i18n'
import type { GameClientState } from './client'
import { GamePanel, GamePanelView, type GameActions } from './GamePanel'
import type { RoomView } from './schema'
import { room, snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({
  GameArena: ({ label, waitingText, enabled }: GameArenaProps) => (
    <div role="region" aria-label={label} data-enabled={String(enabled)}>{waitingText}</div>
  ),
}))

function actions(): GameActions {
  return {
    admit: vi.fn(() => true), ready: vi.fn(() => true), rematch: vi.fn(() => true),
    movement: vi.fn(() => true), placeBomb: vi.fn(() => true), leave: vi.fn(() => true),
  }
}

function state(currentRoom: RoomView, withSnapshot = true): GameClientState {
  const physical = snapshot(currentRoom.revision, currentRoom.roundId)
  physical.room = currentRoom
  return {
    connection: 'authenticated', room: currentRoom, snapshot: withSnapshot ? physical : null,
    youSeat: 'one', failure: null, admissionPending: false, leavePending: false,
  }
}

function renderPanel(currentRoom: RoomView, currentActions = actions(), withSnapshot = true) {
  render(
    <MantineProvider env="test">
      <GamePanelView state={state(currentRoom, withSnapshot)} actions={currentActions} />
    </MantineProvider>,
  )
  return currentActions
}

describe('v3 GamePanel authoritative match presentation', () => {
  beforeEach(async () => { await i18n.changeLanguage('en') })

  it('exposes the dormant hook-backed panel at the lobby', () => {
    render(<MantineProvider env="test"><GamePanel /></MantineProvider>)
    expect(screen.getByRole('button', { name: 'Create room' })).toBeVisible()
  })

  it('shows scores, round number, and the completed-round result during its hold', () => {
    const current = room(300, { type: 'result', nextCountdownAt: 480 }, 4)
    current.tick = 330
    current.roundNumber = 2
    current.scores = { one: 1, two: 0 }
    current.lastRound = {
      roundId: 4, roundNumber: 2, endedAt: 300,
      result: { type: 'winner', seat: 'one' },
    }
    renderPanel(current)

    expect(screen.getByText('Player one · 1')).toBeVisible()
    expect(screen.getByText('0 · Player two')).toBeVisible()
    expect(screen.getByText(/Round 2/)).toBeVisible()
    expect(screen.getByRole('status')).toHaveTextContent('Player one wins the round.')
    expect(screen.getByLabelText('Phase time remaining')).toHaveTextContent('0:03')
  })

  it('uses the match outcome independently of the last completed round and physical actors', () => {
    const current = room(500, {
      type: 'matchResult', endedAt: 500, rematchAt: 680,
      result: { type: 'winner', seat: 'one', reason: 'forfeit' },
    }, 8)
    current.roundNumber = 3
    current.scores = { one: 1, two: 1 }
    current.lastRound = {
      roundId: 6, roundNumber: 2, endedAt: 400,
      result: { type: 'winner', seat: 'two' },
    }
    const physical = snapshot(500, 8)
    physical.state.actors[0].alive = false
    render(
      <MantineProvider env="test">
        <GamePanelView state={{ ...state(current), snapshot: physical }} actions={actions()} />
      </MantineProvider>,
    )

    expect(screen.getByRole('status')).toHaveTextContent('Player one wins the match by forfeit.')
    expect(screen.queryByText('Player two wins the round.')).not.toBeInTheDocument()
    expect(screen.getByText('Rematch opens after the result hold.')).toBeVisible()
  })

  it('shows a vacated forfeit seat as open despite a frozen live actor', () => {
    const current = room(500, {
      type: 'matchResult', endedAt: 500, rematchAt: 680,
      result: { type: 'winner', seat: 'one', reason: 'forfeit' },
    }, 8)
    current.seats[1].occupied = false
    const frozen = snapshot(500, 8)
    frozen.state.actors[1].alive = true
    frozen.room = current
    render(
      <MantineProvider env="test">
        <GamePanelView state={{ ...state(current), snapshot: frozen }} actions={actions()} />
      </MantineProvider>,
    )

    expect(screen.getByText('Player one wins the match by forfeit.')).toBeVisible()
    expect(screen.getByText('Waiting for a player')).toBeVisible()
    expect(screen.queryByRole('button', { name: 'Play again' })).not.toBeInTheDocument()
    expect(screen.getByText('Both players must be present for a rematch.')).toBeVisible()
  })

  it('keeps a departed seat open after a final round even when its frozen actor is eliminated', () => {
    const current = room(680, {
      type: 'matchResult', endedAt: 500, rematchAt: 680,
      result: { type: 'winner', seat: 'one', reason: 'roundWins' },
    }, 8)
    current.tick = 680
    current.seats[1].occupied = false
    const frozen = snapshot(680, 8)
    frozen.state.actors[1].alive = false
    frozen.room = current
    render(
      <MantineProvider env="test">
        <GamePanelView state={{ ...state(current), snapshot: frozen }} actions={actions()} />
      </MantineProvider>,
    )

    expect(screen.getByText('Waiting for a player')).toBeVisible()
    expect(screen.queryByText('Eliminated')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Play again' })).not.toBeInTheDocument()
    expect(screen.getByText('Both players must be present for a rematch.')).toBeVisible()
  })

  it('enables a withdrawable rematch vote only at the authoritative tick with both seats occupied', async () => {
    const user = userEvent.setup()
    const current = room(9, {
      type: 'matchResult', endedAt: 1, rematchAt: 9,
      result: { type: 'winner', seat: 'two', reason: 'roundWins' },
    })
    current.tick = 9
    const currentActions = renderPanel(current)
    await user.click(screen.getByRole('button', { name: 'Play again' }))
    expect(currentActions.rematch).toHaveBeenCalledWith(true)

    current.seats[0].rematchReady = true
    renderPanel(current, currentActions)
    await user.click(screen.getByRole('button', { name: 'Withdraw rematch vote' }))
    expect(currentActions.rematch).toHaveBeenLastCalledWith(false)
  })

  it('offers readiness only before the initial round and shows fresh match facts after consensus', async () => {
    const user = userEvent.setup()
    const initial = room(1, { type: 'countdown', kind: 'initial', endsAt: 181 })
    const initialActions = renderPanel(initial, actions(), false)
    await user.click(screen.getByRole('button', { name: 'Ready' }))
    expect(initialActions.ready).toHaveBeenCalledWith(true)

    cleanup()
    const interRound = room(2, { type: 'countdown', kind: 'interRound', endsAt: 182 }, 2)
    renderPanel(interRound, actions(), false)
    expect(screen.getAllByText('Next round starting')).not.toHaveLength(0)
    expect(screen.queryAllByRole('button', { name: 'Ready' })).toHaveLength(0)

    cleanup()
    const fresh = room(3, { type: 'waiting' }, 3)
    fresh.roundNumber = 0
    fresh.scores = { one: 0, two: 0 }
    renderPanel(fresh, actions(), false)
    expect(screen.getByText(/Match awaiting first round/)).toBeVisible()
    expect(screen.getAllByRole('button', { name: 'Ready' })).not.toHaveLength(0)
    expect(screen.queryByRole('button', { name: 'Play again' })).not.toBeInTheDocument()
  })

  it('localizes match results and rematch controls in Polish', async () => {
    await i18n.changeLanguage('pl')
    const current = room(181, {
      type: 'matchResult', endedAt: 1, rematchAt: 181,
      result: { type: 'winner', seat: 'two', reason: 'roundWins' },
    })
    current.tick = 181
    renderPanel(current)
    expect(screen.getByRole('status')).toHaveTextContent('Gracz drugi wygrywa mecz trzema rundami.')
    expect(screen.getByRole('button', { name: 'Zagraj ponownie' })).toBeVisible()
  })
})
