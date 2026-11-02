import { MantineProvider } from '@mantine/core'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import type { GameArenaProps } from '../arena/types'
import i18n from '../../i18n'
import type { GameClientState } from './client'
import { GamePanelView, type GameActions } from './GamePanel'
import { snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({
  GameArena: ({ label, enabled }: GameArenaProps) => (
    <div role="region" aria-label={label} data-enabled={String(enabled)} />
  ),
}))

beforeEach(async () => i18n.changeLanguage('en'))

it('keeps recovery visible and controls disabled while a same-socket resume is pending', () => {
  const current = snapshot(4)
  current.room.phase = { type: 'waiting' }
  current.room.roundNumber = 0
  const state: GameClientState = {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: true, leavePending: false, settingsPending: false, reconnectAttempt: 1,
    admissionBaseline: { roomId: current.room.roomId, revision: 3, tick: 3, physicalTick: 1 },
  }
  const actions: GameActions = {
    admit: vi.fn(), ready: vi.fn(), updateSettings: vi.fn(), rematch: vi.fn(), movement: vi.fn(),
    placeBomb: vi.fn(), leave: vi.fn(),
  }
  render(<MantineProvider env="test"><GamePanelView state={state} actions={actions} /></MantineProvider>)

  expect(screen.getByRole('alert')).toHaveTextContent('Controls will return after a fresh game state arrives.')
  expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute('data-enabled', 'false')
  expect(screen.getByRole('button', { name: 'Leave room' })).toBeDisabled()
  for (const select of screen.getAllByRole('combobox')) expect(select).toBeDisabled()
  expect(screen.getByRole('checkbox', { name: 'Kick' })).toBeDisabled()
  expect(screen.getByRole('button', { name: 'Restore match defaults' })).toBeDisabled()
  expect(screen.getByRole('button', { name: 'Classic settings' })).toBeDisabled()
  expect(screen.getByRole('button', { name: 'Apply rules' })).toBeDisabled()
})

it('allows a throttled lobby admission to be retried while the socket stays authenticated', async () => {
  const user = userEvent.setup()
  const admit = vi.fn(() => true)
  const state: GameClientState = {
    connection: 'authenticated', room: null, snapshot: null, youSeat: null, failure: 'rateLimited',
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: null,
  }
  const actions: GameActions = {
    admit, ready: vi.fn(), updateSettings: vi.fn(), rematch: vi.fn(), movement: vi.fn(),
    placeBomb: vi.fn(), leave: vi.fn(),
  }
  render(<MantineProvider env="test"><GamePanelView state={state} actions={actions} /></MantineProvider>)

  expect(screen.getByRole('alert')).toHaveTextContent('Commands arrived too quickly.')
  await user.click(screen.getByRole('button', { name: 'Create room' }))
  expect(admit).toHaveBeenCalledWith({ type: 'create' })
  await user.type(screen.getByRole('textbox', { name: 'Room code' }), 'ABCDEFGH')
  await user.click(screen.getByRole('button', { name: 'Join room' }))
  expect(admit).toHaveBeenCalledWith({ type: 'join', roomCode: 'ABCDEFGH' })
})
