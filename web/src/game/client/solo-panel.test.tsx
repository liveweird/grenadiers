import { MantineProvider } from '@mantine/core'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import i18n from '../../i18n'
import type { GameClientState } from './client'
import { initialGameClientState } from './client'
import { GamePanelView, type GameActions } from './GamePanel'
import { room } from './testSupport'

vi.mock('../arena/GameArena', () => ({ GameArena: () => <div /> }))

const actions = (): GameActions => ({
  admit: vi.fn(() => true), ready: vi.fn(() => true), updateSettings: vi.fn(() => true),
  rematch: vi.fn(() => true), movement: vi.fn(() => true), placeBomb: vi.fn(() => true),
  useEquipment: vi.fn(() => true), leave: vi.fn(() => true),
})

function soloState(): GameClientState {
  const current = room(200, {
    type: 'matchResult', endedAt: 20, rematchAt: 200,
    result: { type: 'winner', seat: 'two', reason: 'roundWins' },
  })
  current.mode = { type: 'solo', difficulty: 'standard' }
  current.scores.two = 3
  current.roundNumber = 3
  current.seats[1] = {
    seat: 'two', controller: { type: 'cpu', difficulty: 'standard' }, occupied: true,
    ready: false, rematchReady: true, connection: { type: 'serverControlled' },
  }
  return { ...initialGameClientState(), connection: 'authenticated', room: current, youSeat: 'one' }
}

beforeEach(async () => { localStorage.clear(); await i18n.changeLanguage('en') })
afterEach(cleanup)

it('offers an explicit solo admission and disables it during an outstanding admission', () => {
  const controls = actions()
  const state = initialGameClientState()
  const view = render(<MantineProvider env="test"><GamePanelView state={state} actions={controls} /></MantineProvider>)
  fireEvent.click(screen.getByRole('button', { name: 'Play solo against CPU' }))
  expect(controls.admit).toHaveBeenCalledWith({ type: 'create', mode: { type: 'solo', difficulty: 'standard' } })
  view.rerender(<MantineProvider env="test"><GamePanelView state={{ ...state, admissionPending: true }} actions={controls} /></MantineProvider>)
  expect(screen.getByRole('button', { name: 'Play solo against CPU' })).toBeDisabled()
})

it.each([
  ['en', 'Solo match', 'CPU opponent wins the match with 3 round wins.', 'Play again'],
  ['pl', 'Mecz solo', 'Komputer wygrywa mecz (wygrane rundy: 3).', 'Zagraj ponownie'],
])('labels the CPU result and enables human rematch in %s without offering an invitation', async (language, label, outcome, rematch) => {
  await i18n.changeLanguage(language)
  const controls = actions()
  render(<MantineProvider env="test"><GamePanelView state={soloState()} actions={controls} /></MantineProvider>)
  expect(screen.getByRole('region', { name: label })).toBeVisible()
  expect(screen.getByText(outcome)).toBeVisible()
  expect(screen.queryByText('ABCDEFGH')).not.toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: rematch }))
  expect(controls.rematch).toHaveBeenCalledWith(true)
})
