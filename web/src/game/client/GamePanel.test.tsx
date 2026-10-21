import { MantineProvider } from '@mantine/core'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import type { GameArenaProps } from '../arena/types'
import i18n from '../../i18n'
import type { GameClientState } from './client'
import { GamePanel, GamePanelView, type GameActions } from './GamePanel'
import { room, snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({
  GameArena: ({ label, enabled, bindings, focusHint, snapshot: current, onDirection, onBomb }: GameArenaProps) => (
    <div role="region" aria-label={label} data-enabled={String(enabled)} data-world={String(Boolean(current))}
      data-north={bindings?.north.join(',')} data-bomb={bindings?.bomb.join(',')} data-focus-hint={focusHint}>
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
    admissionBaseline: {
      roomId: current.room.roomId, revision: current.room.revision, tick: current.room.tick,
      physicalTick: current.state.physicalTick,
    },
    admissionPending: connection !== 'authenticated', leavePending: false, settingsPending: false,
    reconnectAttempt: connection === 'reconnecting' ? 1 : 0,
  }
}

function renderPanel(current: GameClientState) {
  render(<MantineProvider env="test"><GamePanelView state={current} actions={actions()} /></MantineProvider>)
}

beforeEach(async () => { localStorage.clear(); await i18n.changeLanguage('en') })

describe('v9 reconnect presentation', () => {
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
})

describe('v9 private-state presentation', () => {
  it('removes hidden rival stats and accessibility details until a fresh reveal', () => {
    const current = state()
    const view = render(<MantineProvider env="test">
      <GamePanelView state={current} actions={actions()} />
    </MantineProvider>)
    expect(screen.getByLabelText('Player two upgrades')).toBeVisible()

    current.snapshot!.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    view.rerender(<MantineProvider env="test">
      <GamePanelView state={{ ...current }} actions={actions()} />
    </MantineProvider>)
    expect(screen.queryByLabelText('Player two upgrades')).not.toBeInTheDocument()

    current.snapshot!.state.actors[1] = {
      type: 'visible', id: 2, seat: 'two', position: { x: 4096, y: 3072 }, heading: 'north',
      speedLevel: 2, alive: true, bombCapacity: 3, bombRange: 4, liveBombCount: 1,
      inventory: { equipment: null, weapon: null }, effects: [], damageGraceRemainingTicks: 0, voluntaryActionsFrozen: false,
    }
    view.rerender(<MantineProvider env="test">
      <GamePanelView state={{ ...current }} actions={actions()} />
    </MantineProvider>)
    expect(screen.getByLabelText('Player two upgrades')).toHaveTextContent('Blast range4')
  })

  it('shows bounded authoritative inventory, effect timers, grace, and bomb counts as static stats', () => {
    const current = state()
    const actor = current.snapshot!.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible own actor')
    actor.bombCapacity = 4
    actor.liveBombCount = 3
    actor.inventory = { equipment: 'heart', weapon: 'remoteBomb' }
    actor.effects = [
      { kind: 'shield', remainingTicks: 120 },
      { kind: 'goldenFlame', remainingTicks: 600 },
      { kind: 'shortFuse', remainingTicks: 599 },
      { kind: 'slowFoot', remainingTicks: 600 },
      { kind: 'slowFoot', remainingTicks: 120 },
      { kind: 'shortFlame', remainingTicks: 1 },
    ]
    actor.damageGraceRemainingTicks = 60
    renderPanel(current)

    const stats = screen.getByLabelText('Player one upgrades')
    expect(stats).toHaveTextContent('Bombs available1')
    expect(stats).toHaveTextContent('GearHeart')
    expect(stats).toHaveTextContent('Bomb typeRemote bomb')
    expect(stats).toHaveTextContent('EffectsShield (0:02)')
    expect(stats).toHaveTextContent('Golden flame (0:10)')
    expect(stats).toHaveTextContent('Short fuse (0:10)')
    expect(stats).toHaveTextContent('Slow foot ×2 (1/4 speed; 0:10, 0:02)')
    expect(stats).toHaveTextContent('Short flame (0:01)')
    expect(stats).toHaveTextContent('Damage grace0:01')
    expect(screen.getByLabelText('Protected from damage')).toHaveTextContent('0:01')
    expect(stats).not.toHaveAttribute('role', 'status')
    expect(stats.closest('[aria-live]')).toBeNull()
    expect(screen.getByText('Round in progress')).toHaveAttribute('aria-live', 'polite')
  })

  it('uses the existing bomb binding in the armed Remote detonation hint', () => {
    const current = state()
    const actor = current.snapshot!.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible own actor')
    current.snapshot!.state.bombs = [{
      id: 7, owner: actor.id, cell: { column: 3, row: 1 }, range: 2, placedAt: 1,
      trigger: { type: 'remote' }, blast: 'standard',
    }]
    renderPanel(current)
    expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute(
      'data-focus-hint', 'Remote bomb armed. Select the courtyard and press Space to detonate it.',
    )
    expect(screen.getByLabelText('Player one upgrades')).toHaveTextContent('Bomb typeNone')
  })

  it('announces authoritative Stopwatch freeze without inventing an effect timer', () => {
    const current = state()
    const actor = current.snapshot!.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible own actor')
    actor.voluntaryActionsFrozen = true
    renderPanel(current)
    const stats = screen.getByLabelText('Player one upgrades')
    expect(stats).toHaveTextContent('ActionsFrozen by Stopwatch')
    expect(stats).not.toHaveTextContent('Stopwatch (')
  })

  it('groups all six Slow-foot timers in the eight-field HUD in English and Polish', async () => {
    const current = state()
    const actor = current.snapshot!.state.actors[0]
    if (actor.type !== 'visible') throw new Error('expected visible own actor')
    actor.effects = [600, 540, 420, 300, 180, 60]
      .map((remainingTicks) => ({ kind: 'slowFoot' as const, remainingTicks }))
    const view = render(<MantineProvider env="test">
      <GamePanelView state={current} actions={actions()} />
    </MantineProvider>)
    let stats = screen.getByLabelText('Player one upgrades')
    expect(stats.querySelectorAll('dt')).toHaveLength(8)
    expect(stats).toHaveTextContent('Slow foot ×6 (1/64 speed; 0:10, 0:09, 0:07, 0:05, 0:03, 0:01)')

    await i18n.changeLanguage('pl')
    view.rerender(<MantineProvider env="test">
      <GamePanelView state={{ ...current }} actions={actions()} />
    </MantineProvider>)
    stats = screen.getByLabelText('Ulepszenia: Gracz pierwszy')
    expect(stats.querySelectorAll('dt')).toHaveLength(8)
    expect(stats).toHaveTextContent('Wolna stopa ×6 (prędkość 1/64; 0:10, 0:09, 0:07, 0:05, 0:03, 0:01)')
  })
})

describe('v9 room controls', () => {
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
      enabledItems: ['heart'],
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

  it.each([
    ['en', 1, 'Player one wins the match with one round win.'],
    ['en', 3, 'Player one wins the match with 3 round wins.'],
    ['en', 9, 'Player one wins the match with 9 round wins.'],
    ['pl', 1, 'Gracz pierwszy wygrywa mecz po jednej wygranej rundzie.'],
    ['pl', 3, 'Gracz pierwszy wygrywa mecz (wygrane rundy: 3).'],
    ['pl', 9, 'Gracz pierwszy wygrywa mecz (wygrane rundy: 9).'],
  ] as const)('reports the configured winning target in %s at %s wins', async (language, wins, text) => {
    await i18n.changeLanguage(language)
    const current = state()
    current.room = room(200, {
      type: 'matchResult', endedAt: 20, rematchAt: 200,
      result: { type: 'winner', seat: 'one', reason: 'roundWins' },
    }, wins)
    current.room.settings.winsToFinish = wins
    current.room.scores.one = wins
    current.snapshot!.room = current.room
    renderPanel(current)
    expect(screen.getByRole('status')).toHaveTextContent(text)
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
