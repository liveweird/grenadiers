import { MantineProvider } from '@mantine/core'
import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, expect, it, vi } from 'vitest'
import i18n from '../../i18n'
import type { GameClientState } from './client'
import { initialGameClientState } from './client'
import { GamePanelView, type GameActions } from './GamePanel'
import { snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({
  GameArena: ({ label, enabled }: { label: string; enabled?: boolean }) =>
    <div role="region" aria-label={label} data-enabled={String(enabled)} />,
}))

const actions = (): GameActions => ({
  admit: vi.fn(() => true), ready: vi.fn(() => true), updateSettings: vi.fn(() => true),
  rematch: vi.fn(() => true), movement: vi.fn(() => true), placeBomb: vi.fn(() => true),
  leave: vi.fn(() => true),
})

const show = (state: GameClientState): void => {
  render(<MantineProvider env="test"><GamePanelView state={state} actions={actions()} /></MantineProvider>)
}

beforeEach(async () => i18n.changeLanguage('en'))

it('keeps lobby actions available and preserves a custom-match draft while hidden', async () => {
  const user = userEvent.setup()
  show(initialGameClientState())
  const toggle = screen.getByRole('button', { name: 'Open settings' })
  expect(screen.getByRole('button', { name: 'Create room' })).toBeVisible()
  expect(screen.getByRole('button', { name: 'Play solo against CPU' })).toBeVisible()
  expect(screen.getByRole('button', { name: 'Join room' })).toBeVisible()
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  expect(screen.queryByRole('combobox', { name: 'Players' })).not.toBeInTheDocument()

  await user.click(toggle)
  await waitFor(() => expect(screen.getByRole('button', { name: 'Close settings' })).toHaveFocus())
  fireEvent.change(screen.getByRole('combobox', { name: 'Players' }), { target: { value: '4' } })
  await user.tab()
  expect(screen.getByRole('dialog')).toContainElement(document.activeElement as HTMLElement)
  await user.keyboard('{Escape}')
  await waitFor(() => expect(toggle).toHaveFocus())
  expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
  expect(screen.queryByRole('combobox', { name: 'Players' })).not.toBeInTheDocument()

  fireEvent.click(toggle)
  expect(screen.getByRole('combobox', { name: 'Players' })).toHaveValue('4')
})

it('keeps room actions available and preserves an unapplied rules draft', () => {
  const current = snapshot(20)
  current.room.phase = { type: 'waiting' }
  current.room.roundNumber = 0
  show({
    ...initialGameClientState(), connection: 'authenticated', room: current.room,
    snapshot: null, youSeat: 'one',
  })
  expect(screen.getByRole('button', { name: 'Ready' })).toBeVisible()
  expect(screen.getByRole('button', { name: 'Leave room' })).toBeVisible()

  fireEvent.click(screen.getByRole('button', { name: 'Open settings' }))
  fireEvent.change(screen.getByLabelText('Round wins needed'), { target: { value: '1' } })
  fireEvent.click(screen.getByRole('button', { name: 'Close settings' }))
  expect(screen.queryByRole('combobox', { name: 'Round wins needed' })).not.toBeInTheDocument()
  fireEvent.click(screen.getByRole('button', { name: 'Open settings' }))
  expect(screen.getByLabelText('Round wins needed')).toHaveValue('1')
})

it('disables arena input while settings have focus', () => {
  const current = snapshot(20)
  show({ ...initialGameClientState(), connection: 'authenticated', room: current.room,
    snapshot: current, youSeat: 'one' })
  const arena = screen.getByRole('region', { name: 'Grenadiers courtyard' })
  expect(arena).toHaveAttribute('data-enabled', 'true')
  fireEvent.click(screen.getByRole('button', { name: 'Open settings' }))
  expect(arena).toHaveAttribute('data-enabled', 'false')
})
