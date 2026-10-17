import { MantineProvider } from '@mantine/core'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameArenaProps } from '../arena/types'
import i18n from '../../i18n'
import type { GameClientState } from './client'
import { GamePanel, GamePanelView, type GameActions } from './GamePanel'
import { room, snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({
  GameArena: ({ label, enabled, bindings, snapshot: current, onDirection, onBomb }: GameArenaProps) => (
    <div role="region" aria-label={label} data-enabled={String(enabled)} data-world={String(Boolean(current))}
      data-north={bindings?.north.join(',')} data-bomb={bindings?.bomb.join(',')}>
      <button onClick={() => onDirection('east')}>Move in test arena</button>
      <button onClick={() => onDirection(null)}>Release in test arena</button>
      <button onClick={() => onBomb()}>Bomb in test arena</button>
    </div>
  ),
}))

function actions(): GameActions {
  return {
    admit: vi.fn(() => true), ready: vi.fn(() => true), updateSettings: vi.fn(() => true),
    rematch: vi.fn(() => true),
    movement: vi.fn(() => true), placeBomb: vi.fn(() => true), leave: vi.fn(() => true),
  }
}

function state(connection: GameClientState['connection'] = 'authenticated'): GameClientState {
  const current = snapshot(20)
  return {
    connection, room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionBaseline: { roomId: current.room.roomId, revision: current.room.revision, tick: current.room.tick },
    admissionPending: connection !== 'authenticated', leavePending: false, settingsPending: false,
    reconnectAttempt: connection === 'reconnecting' ? 1 : 0,
  }
}

function renderPanel(current: GameClientState) {
  render(<MantineProvider env="test"><GamePanelView state={current} actions={actions()} /></MantineProvider>)
}

describe('v6 reconnect presentation', () => {
  beforeEach(async () => { localStorage.clear(); await i18n.changeLanguage('en') })

  it('exposes the dormant hook-backed panel without activating it in App', () => {
    render(<MantineProvider env="test"><GamePanel /></MantineProvider>)
    expect(screen.getByRole('button', { name: 'Create room' })).toBeVisible()
  })

  it('keeps the last courtyard visible and disables input while reconnecting', () => {
    const current = state('reconnecting')
    current.room!.seats[0].connection = { type: 'disconnected', expiresAt: 620 }
    current.room!.tick = 20
    renderPanel(current)
    const arena = screen.getByRole('region', { name: 'Grenadiers courtyard' })
    expect(arena).toHaveAttribute('data-world', 'true')
    expect(arena).toHaveAttribute('data-enabled', 'false')
    expect(screen.getByRole('alert')).toHaveTextContent('Controls will return after a fresh game state arrives.')
    expect(screen.getByRole('timer')).toHaveTextContent('0:10')
  })

  it('enables input only after a connected admitted baseline', () => {
    renderPanel(state('authenticated'))
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-enabled', 'true')
  })

  it('blocks readiness and rule edits while an update awaits authority', () => {
    const current = state()
    current.room = room(20, { type: 'waiting' }, 1)
    current.snapshot = null
    current.settingsPending = true
    renderPanel(current)
    expect(screen.getByRole('button', { name: 'Ready' })).toBeDisabled()
    expect(screen.getByLabelText('Starting speed')).toBeDisabled()
  })

  it('forwards a complete settings draft through the action receiver', () => {
    const current = state()
    current.room = room(20, { type: 'waiting' }, 1)
    current.snapshot = null
    const received: unknown[] = []
    const currentActions = {
      ...actions(),
      updateSettings(settings: unknown) { received.push([this, settings]); return true },
    }
    render(<MantineProvider env="test"><GamePanelView state={current} actions={currentActions} /></MantineProvider>)
    fireEvent.change(screen.getByLabelText('Round wins needed'), { target: { value: '1' } })
    fireEvent.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(received).toEqual([[currentActions, {
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 1, bombFuseTicks: 180,
    }]])
  })

  it('preserves the action receiver for arena movement, release and bombs', () => {
    const received: unknown[] = []
    const currentActions = {
      ...actions(),
      movement(direction: 'east' | null) { received.push([this, direction]); return true },
      placeBomb() { received.push([this, 'bomb']); return true },
    }
    render(<MantineProvider env="test"><GamePanelView state={state()} actions={currentActions} /></MantineProvider>)
    fireEvent.click(screen.getByRole('button', { name: 'Move in test arena' }))
    fireEvent.click(screen.getByRole('button', { name: 'Release in test arena' }))
    fireEvent.click(screen.getByRole('button', { name: 'Bomb in test arena' }))
    expect(received).toEqual([[currentActions, 'east'], [currentActions, null], [currentActions, 'bomb']])
  })

  it('explains a completed-round hold that waits beyond its deadline for connection', () => {
    const current = state()
    current.room = room(400, { type: 'result', holdUntil: 300 }, 3)
    current.room.tick = 400
    current.room.roundNumber = 2
    current.room.scores.one = 1
    current.room.lastRound = {
      roundId: 3, roundNumber: 2, endedAt: 120, result: { type: 'winner', seat: 'one' },
    }
    current.room.seats[1].connection = { type: 'disconnected', expiresAt: 500 }
    current.snapshot!.room = current.room
    renderPanel(current)
    expect(screen.getByText('The next round will wait for both players to reconnect.')).toBeVisible()
    expect(screen.getByText('Player one wins the round.')).toBeVisible()
  })

  it('requires connected seats for rematch controls and localizes grace in Polish', async () => {
    await i18n.changeLanguage('pl')
    const current = state()
    current.room = room(200, {
      type: 'matchResult', endedAt: 20, rematchAt: 200,
      result: { type: 'winner', seat: 'one', reason: 'roundWins' },
    }, 3)
    current.room.seats[1].connection = { type: 'disconnected', expiresAt: 260 }
    current.snapshot!.room = current.room
    renderPanel(current)
    expect(screen.getByText('Do rewanżu potrzebni są obaj gracze.')).toBeVisible()
    expect(screen.queryByRole('button', { name: 'Zagraj ponownie' })).not.toBeInTheDocument()
    expect(screen.getByText(/Połącz się ponownie/)).toBeVisible()
    cleanup()
  })

  it('disables an otherwise eligible rematch while offline', () => {
    const current = state('disconnected')
    current.admissionPending = false
    current.room = room(200, {
      type: 'matchResult', endedAt: 20, rematchAt: 200,
      result: { type: 'winner', seat: 'one', reason: 'roundWins' },
    }, 3)
    current.snapshot!.room = current.room
    renderPanel(current)
    expect(screen.getByRole('button', { name: 'Play again' })).toBeDisabled()
  })

  it('captures unique physical keys, cancels with Escape, and restores defaults', () => {
    renderPanel(state())
    const arena = screen.getByRole('region', { name: 'Grenadiers courtyard' })
    fireEvent.click(screen.getByRole('button', { name: 'Keyboard controls' }))
    expect(arena).toHaveAttribute('data-enabled', 'false')
    for (const action of ['North', 'East', 'South', 'West', 'Bomb']) {
      expect(screen.getByRole('button', { name: `Change ${action}` })).toBeVisible()
    }
    fireEvent.click(screen.getByRole('button', { name: 'Change North' }))
    expect(screen.getByRole('button', { name: 'Press a key for North' })).toHaveAttribute('aria-pressed', 'true')
    for (const rejected of [
      new KeyboardEvent('keydown', { code: 'F5', key: 'F5', cancelable: true }),
      new KeyboardEvent('keydown', { code: 'KeyL', key: 'l', ctrlKey: true, cancelable: true }),
      new KeyboardEvent('keydown', { code: 'KeyR', key: 'r', ctrlKey: true, cancelable: true }),
    ]) {
      window.dispatchEvent(rejected)
      expect(rejected.defaultPrevented).toBe(false)
    }
    fireEvent.keyDown(window, { code: 'ArrowRight', key: 'ArrowRight' })
    expect(screen.getByRole('alert')).toHaveTextContent('That key already controls East.')
    expect(arena).toHaveAttribute('data-north', 'ArrowUp,KeyW')
    fireEvent.keyDown(window, { code: 'KeyI', key: 'i' })
    expect(arena).toHaveAttribute('data-north', 'KeyI')
    expect(screen.getByText(/North: I/)).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Change East' }))
    fireEvent.keyDown(window, { code: 'Escape', key: 'Escape' })
    expect(screen.queryByText('Press an allowed key, or Escape to cancel. Tab still moves focus.')).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Restore defaults' }))
    expect(arena).toHaveAttribute('data-north', 'ArrowUp,KeyW')
    fireEvent.click(screen.getByRole('button', { name: 'Done' }))
    expect(arena).toHaveAttribute('data-enabled', 'true')
  })

  it('persists a remap across remount and localizes settings in Polish', async () => {
    const view = render(<MantineProvider env="test"><GamePanelView state={state()} actions={actions()} /></MantineProvider>)
    fireEvent.click(screen.getByRole('button', { name: 'Keyboard controls' }))
    fireEvent.click(screen.getByRole('button', { name: 'Change Bomb' }))
    fireEvent.keyDown(window, { code: 'Enter', key: 'Enter' })
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-bomb', 'Enter')
    view.unmount()
    await i18n.changeLanguage('pl')
    render(<MantineProvider env="test"><GamePanelView state={state()} actions={actions()} /></MantineProvider>)
    expect(screen.getByRole('region', { name: 'Dziedziniec Grenadiers' })).toHaveAttribute('data-bomb', 'Enter')
    fireEvent.click(screen.getByRole('button', { name: 'Sterowanie klawiaturą' }))
    expect(screen.getByRole('dialog', { name: 'Sterowanie klawiaturą' })).toHaveTextContent('Północ')
    for (const action of ['Północ', 'Wschód', 'Południe', 'Zachód', 'Bomba']) {
      expect(screen.getByRole('button', { name: `Zmień: ${action}` })).toBeVisible()
    }
    fireEvent.click(screen.getByRole('button', { name: 'Zmień: Północ' }))
    expect(screen.getByRole('button', { name: 'Naciśnij klawisz dla akcji: Północ' }))
      .toHaveAttribute('aria-pressed', 'true')
    expect(screen.getByText(/Bomba: Enter/)).toBeVisible()
  })
})
