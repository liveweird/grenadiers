import { MantineProvider } from '@mantine/core'
import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import en from '../../locales/en/game.json'
import { GameplaySettingsPanel } from './GameplaySettingsPanel'
import { room } from './testSupport'

describe('gameplay settings locks', () => {
  it.each([false, true])('explains a null grant and disables controls when ready=%s', (ready) => {
    const locked = room(3)
    locked.settingsEditor = null
    locked.seats[0].ready = ready
    render(<MantineProvider env="test"><GameplaySettingsPanel room={locked} youSeat="one"
      pending={false} copy={en} onApply={() => true} /></MantineProvider>)
    expect(screen.getByText('These match rules are read-only.')).toBeVisible()
    expect(screen.getByLabelText('Starting speed')).toBeDisabled()
    expect(screen.getByLabelText('Turning tolerance')).toBeDisabled()
    expect(screen.getByLabelText('Round wins needed')).toBeDisabled()
    expect(screen.getByLabelText('Bomb fuse')).toBeDisabled()
    expect(screen.getAllByRole('checkbox').every((control) => control.hasAttribute('disabled'))).toBe(true)
    for (const name of ['All on', 'All off', 'Restore match defaults', 'Classic settings', 'Apply rules']) {
      expect(screen.getByRole('button', { name })).toBeDisabled()
    }
  })

  it.each(['editor', 'disconnected', 'match', 'pending'] as const)(
    'locks bulk actions when blocked by %s state', (lock) => {
      const locked = room(3)
      if (lock === 'editor') locked.settingsEditor = 'two'
      if (lock === 'disconnected') locked.seats[0].connection = { type: 'disconnected', expiresAt: 20 }
      if (lock === 'match') locked.phase = { type: 'running' }
      render(<MantineProvider env="test"><GameplaySettingsPanel room={locked} youSeat="one"
        pending={lock === 'pending'} copy={en} onApply={() => true} /></MantineProvider>)
      expect(screen.getByRole('button', { name: 'All on' })).toBeDisabled()
      expect(screen.getByRole('button', { name: 'All off' })).toBeDisabled()
    },
  )

  it('locks all item rules with the rest of the lobby settings', () => {
    const locked = room(3)
    locked.seats[0].ready = true
    render(<MantineProvider env="test"><GameplaySettingsPanel room={locked} youSeat="one"
      pending={false} copy={en} onApply={() => true} /></MantineProvider>)
    expect(screen.getByRole('checkbox', { name: 'Heart' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Wall pass' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Landmine' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Shield' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Golden flame' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Short fuse' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Diarrhea' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Slow foot' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Short flame' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Reverse controls' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Teleporter' })).toBeDisabled()
    expect(screen.getByRole('checkbox', { name: 'Oil barrel' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'All on' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'All off' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Restore match defaults' })).toBeDisabled()
    expect(screen.getByRole('button', { name: 'Classic settings' })).toBeDisabled()
  })
})
