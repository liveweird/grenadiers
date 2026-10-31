import { MantineProvider } from '@mantine/core'
import { cleanup, fireEvent, render, screen } from '@testing-library/react'
import { afterEach, beforeEach, expect, it, vi } from 'vitest'
import i18n from '../../i18n'
import { initialGameClientState } from './client'
import { GamePanelView, type GameActions } from './GamePanel'
import { snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({ GameArena: () => <div /> }))

function actions(): GameActions {
  return {
    admit: vi.fn(() => true), ready: vi.fn(() => true), updateSettings: vi.fn(() => true),
    rematch: vi.fn(() => true), movement: vi.fn(() => true), placeBomb: vi.fn(() => true),
    useEquipment: vi.fn(() => true), leave: vi.fn(() => true),
  }
}

beforeEach(async () => { await i18n.changeLanguage('en') })
afterEach(cleanup)

it('creates an explicit four-controller mixed configuration while keeping player one human', () => {
  const controls = actions()
  render(<MantineProvider env="test">
    <GamePanelView state={initialGameClientState()} actions={controls} />
  </MantineProvider>)
  fireEvent.change(screen.getByRole('combobox', { name: 'Players' }), { target: { value: '4' } })
  fireEvent.change(screen.getByRole('combobox', { name: 'Board size' }), { target: { value: '11x9' } })
  fireEvent.change(screen.getByRole('combobox', { name: 'Player 2 controller' }), { target: { value: 'cpu' } })
  fireEvent.change(screen.getByRole('combobox', { name: 'Player 4 controller' }), { target: { value: 'cpu' } })
  const difficulties = screen.getAllByRole('combobox', { name: 'CPU difficulty' })
  fireEvent.change(difficulties.at(-1)!, { target: { value: 'easy' } })
  fireEvent.click(screen.getByRole('button', { name: 'Create custom match' }))
  expect(controls.admit).toHaveBeenCalledWith({
    type: 'create', configuration: {
      controllers: ['human', 'cpu', 'human', 'cpu'], cpuDifficulty: 'easy',
      board: { width: 11, height: 9 },
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
        spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 } },
    },
  })
})

it.each([
  ['2', '15x13'], ['2', '13x11'], ['2', '11x9'],
  ['3', '15x13'], ['3', '13x11'], ['3', '11x9'],
  ['4', '15x13'], ['4', '13x11'], ['4', '11x9'],
] as const)('creates a %s-player custom match on the %s board preset', (players, board) => {
  const controls = actions()
  render(<MantineProvider env="test">
    <GamePanelView state={initialGameClientState()} actions={controls} />
  </MantineProvider>)
  fireEvent.change(screen.getByRole('combobox', { name: 'Players' }), { target: { value: players } })
  fireEvent.change(screen.getByRole('combobox', { name: 'Board size' }), { target: { value: board } })
  fireEvent.click(screen.getByRole('button', { name: 'Create custom match' }))
  const [width, height] = board.split('x').map(Number)
  expect(controls.admit).toHaveBeenCalledWith(expect.objectContaining({
    configuration: expect.objectContaining({ board: { width, height } }),
  }))
})

it('keeps quick duel and solo on their server-default 15 × 13 profile after changing the custom preset', () => {
  const controls = actions()
  render(<MantineProvider env="test">
    <GamePanelView state={initialGameClientState()} actions={controls} />
  </MantineProvider>)
  fireEvent.change(screen.getByRole('combobox', { name: 'Board size' }), { target: { value: '11x9' } })
  fireEvent.click(screen.getByRole('button', { name: 'Create room' }))
  fireEvent.click(screen.getByRole('button', { name: 'Play solo against CPU' }))
  expect(controls.admit).toHaveBeenNthCalledWith(1, { type: 'create' })
  expect(controls.admit).toHaveBeenNthCalledWith(2, {
    type: 'create', mode: { type: 'solo', difficulty: 'standard' },
  })
})

it.each([
  ['en', 'Custom match', 'Player 3 controller', 'Create custom match'],
  ['pl', 'Mecz niestandardowy', 'Sterowanie gracza 3', 'Utwórz mecz niestandardowy'],
] as const)('localizes the keyboard-operable custom form in %s', async (language, title, controller, create) => {
  await i18n.changeLanguage(language)
  render(<MantineProvider env="test">
    <GamePanelView state={initialGameClientState()} actions={actions()} />
  </MantineProvider>)
  expect(screen.getByText(title)).toBeVisible()
  fireEvent.change(screen.getByRole('combobox', { name: language === 'en' ? 'Players' : 'Gracze' }),
    { target: { value: '3' } })
  expect(screen.getByRole('combobox', { name: controller })).toBeVisible()
  expect(screen.getByRole('button', { name: create })).toBeVisible()
})

it('uses seat four for the local HUD while keeping creator-only settings authority', () => {
  const current = snapshot(2)
  current.room.configuration.controllers = ['human', 'human', 'human', 'human']
  current.room.phase = { type: 'waiting' }
  current.room.roundNumber = 0
  current.room.seats.push(
    { seat: 'three', controller: { type: 'human' }, occupied: true, ready: false,
      rematchReady: false, connection: { type: 'connected' } },
    { seat: 'four', controller: { type: 'human' }, occupied: true, ready: false,
      rematchReady: false, connection: { type: 'connected' } },
  )
  current.room.scores = ['one', 'two', 'three', 'four'].map((seat, wins) => ({
    seat: seat as 'one' | 'two' | 'three' | 'four', wins,
  }))
  const one = current.state.actors[0]!
  const two = current.state.actors[1]!
  if (one.type !== 'visible' || two.type !== 'visible') throw new Error('fixture actors must be visible')
  current.state.actors.push(
    { ...one, id: 3, seat: 'three', position: { x: 512, y: 12_800 } },
    { ...two, id: 4, seat: 'four', position: { x: 14_848, y: 512 } },
  )
  render(<MantineProvider env="test"><GamePanelView state={{
    ...initialGameClientState(), connection: 'authenticated', room: current.room,
    snapshot: current, youSeat: 'four',
  }} actions={actions()} /></MantineProvider>)

  expect(screen.getByRole('list', { name: 'Players' })).toHaveTextContent(/Player four\s+\(you\)/)
  expect(screen.getByRole('group', { name: 'Round wins for Player three' })).toHaveTextContent('2')
  expect(screen.getByRole('group', { name: 'Round wins for Player four' })).toHaveTextContent('3')
  expect(screen.getByLabelText('Your items')).toBeVisible()
  expect(screen.queryByLabelText('Player four upgrades')).not.toBeInTheDocument()
  expect(screen.getByText('Player one can edit while unready.')).toBeVisible()
  expect(screen.getByRole('button', { name: 'Apply rules' })).toBeDisabled()
})

it('keeps the invitation and distinct CPU identity while a mixed room has a human vacancy', () => {
  const current = snapshot(2)
  current.room.rulesVersion = 10
  current.room.phase = { type: 'waiting' }
  current.room.roundNumber = 0
  current.room.configuration = {
    controllers: ['human', 'cpu', 'human', 'cpu'], cpuDifficulty: 'standard',
    board: { width: 15, height: 13 },
    environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 },
      spikePits: { enabled: true, warningTicks: 36, raisedTicks: 180 } },
  }
  current.room.seats[1] = {
    seat: 'two', controller: { type: 'cpu', difficulty: 'standard' }, occupied: true,
    ready: true, rematchReady: false, connection: { type: 'serverControlled' },
  }
  current.room.seats.push({
    seat: 'three', controller: { type: 'human' }, occupied: false, ready: false,
    rematchReady: false, connection: { type: 'vacant' },
  }, {
    seat: 'four', controller: { type: 'cpu', difficulty: 'standard' }, occupied: true,
    ready: true, rematchReady: false, connection: { type: 'serverControlled' },
  })
  current.room.scores.push({ seat: 'three', wins: 0 }, { seat: 'four', wins: 0 })
  render(<MantineProvider env="test"><GamePanelView state={{
    ...initialGameClientState(), connection: 'authenticated', room: current.room, youSeat: 'one',
  }} actions={actions()} /></MantineProvider>)

  expect(screen.getByRole('region', { name: 'Multiplayer match' })).toBeVisible()
  expect(screen.getByText('CPU difficulty: Standard')).toBeVisible()
  expect(screen.getByText('Board: 15 × 13')).toBeVisible()
  expect(screen.queryByRole('combobox', { name: 'Board size' })).not.toBeInTheDocument()
  expect(screen.getByLabelText('Room invitation code')).toHaveTextContent('ABCDEFGH')
  expect(screen.getByRole('group', { name: 'Round wins for Player two (CPU)' })).toBeVisible()
  expect(screen.getByRole('group', { name: 'Round wins for Player three' })).toBeVisible()
  expect(screen.getByRole('group', { name: 'Round wins for Player four (CPU)' })).toBeVisible()
  expect(screen.getByRole('list', { name: 'Players' })).toHaveTextContent('Player two (CPU)')
  expect(screen.getByRole('list', { name: 'Players' })).toHaveTextContent('Player three')
  expect(screen.getByRole('list', { name: 'Players' })).toHaveTextContent('Player four (CPU)')
})
