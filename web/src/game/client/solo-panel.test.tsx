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

function soloState(difficulty: 'easy' | 'standard' = 'standard'): GameClientState {
  const current = room(200, {
    type: 'matchResult', endedAt: 20, rematchAt: 200,
    result: { type: 'winner', seat: 'two', reason: 'roundWins' },
  })
  current.mode = { type: 'solo', difficulty }
  current.scores.two = 3
  current.roundNumber = 3
  current.seats[1] = {
    seat: 'two', controller: { type: 'cpu', difficulty }, occupied: true,
    ready: false, rematchReady: true, connection: { type: 'serverControlled' },
  }
  return { ...initialGameClientState(), connection: 'authenticated', room: current, youSeat: 'one' }
}

beforeEach(async () => { localStorage.clear(); await i18n.changeLanguage('en') })
afterEach(cleanup)

it('defaults solo admission to Standard, accepts Easy, and disables the selector while busy', () => {
  const controls = actions()
  const state = initialGameClientState()
  const view = render(<MantineProvider env="test"><GamePanelView state={state} actions={controls} /></MantineProvider>)
  const difficulty = screen.getByRole('combobox', { name: 'CPU difficulty' })
  fireEvent.change(difficulty, { target: { value: 'unsupported' } })
  fireEvent.click(screen.getByRole('button', { name: 'Play solo against CPU' }))
  expect(controls.admit).toHaveBeenCalledWith({ type: 'create', mode: { type: 'solo', difficulty: 'standard' } })
  fireEvent.change(difficulty, { target: { value: 'easy' } })
  fireEvent.click(screen.getByRole('button', { name: 'Play solo against CPU' }))
  expect(controls.admit).toHaveBeenLastCalledWith({ type: 'create', mode: { type: 'solo', difficulty: 'easy' } })
  for (const pending of [
    { admissionPending: true },
    { connection: 'connecting' as const },
    { connection: 'authenticating' as const },
    { connection: 'reconnecting' as const },
  ]) {
    view.rerender(<MantineProvider env="test"><GamePanelView state={{ ...state, ...pending }} actions={controls} /></MantineProvider>)
    expect(screen.getByRole('combobox', { name: 'CPU difficulty' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Play solo against CPU' })).toBeDisabled()
  }
})

it.each([
  ['en', 'easy', 'Private solo match · Easy CPU'],
  ['en', 'standard', 'Private solo match · Standard CPU'],
  ['pl', 'easy', 'Prywatny mecz solo · Łatwy komputer'],
  ['pl', 'standard', 'Prywatny mecz solo · Standardowy komputer'],
] as const)('renders the server-provided %s %s difficulty', async (language, difficulty, label) => {
  await i18n.changeLanguage(language)
  render(<MantineProvider env="test"><GamePanelView state={soloState(difficulty)} actions={actions()} /></MantineProvider>)
  expect(screen.getByText(label)).toBeVisible()
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
