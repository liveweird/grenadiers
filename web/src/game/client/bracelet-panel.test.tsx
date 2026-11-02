import { MantineProvider } from '@mantine/core'
import { render, screen } from '@testing-library/react'
import { expect, it, vi } from 'vitest'
import type { GameArenaProps } from '../arena/types'
import { GamePanelView, type GameActions } from './GamePanel'
import { initialGameClientState } from './clientTypes'
import { snapshot } from './testSupport'

vi.mock('../arena/GameArena', () => ({
  GameArena: ({ label, focusHint }: GameArenaProps) =>
    <div role="region" aria-label={label} data-focus-hint={focusHint} />,
}))

it('labels Bracelet as passive equipment without an action hint or cooldown timer', () => {
  const current = snapshot(20)
  const actor = current.state.actors[0]
  if (actor.type !== 'visible') throw new Error('expected visible own actor')
  actor.inventory = { equipment: 'bracelet', weapon: null }
  const state = {
    ...initialGameClientState(), connection: 'authenticated' as const,
    room: current.room, snapshot: current, youSeat: 'one' as const,
  }
  const action = vi.fn(() => true)
  const actions: GameActions = {
    admit: action, ready: action, updateSettings: action, rematch: action,
    movement: action, placeBomb: action, leave: action,
  }
  render(<MantineProvider env="test"><GamePanelView state={state} actions={actions} /></MantineProvider>)
  const stats = screen.getByLabelText('Your items')
  expect(stats).toHaveTextContent('GearBracelet')
  expect(stats).not.toHaveTextContent('Bracelet (')
  expect(screen.getByRole('region', { name: 'Grenadiers courtyard' })).toHaveAttribute(
    'data-focus-hint', 'Select the courtyard to control your player.',
  )
})
