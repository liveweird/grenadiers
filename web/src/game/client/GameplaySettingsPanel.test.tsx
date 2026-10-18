import { MantineProvider } from '@mantine/core'
import { render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import en from '../../locales/en/game.json'
import pl from '../../locales/pl/game.json'
import { GameplaySettingsPanel } from './GameplaySettingsPanel'
import { room } from './testSupport'

describe('gameplay settings panel', () => {
  beforeEach(() => vi.clearAllMocks())

  it('uses player units and applies one complete draft', async () => {
    const apply = vi.fn(() => true)
    const user = userEvent.setup()
    render(<MantineProvider env="test"><GameplaySettingsPanel room={room(1)} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)

    expect(screen.getByRole('heading', { name: 'Match rules', level: 2 })).toBeVisible()
    expect(screen.getByRole('option', { name: '3 cells/s' })).toBeVisible()
    expect(screen.getByRole('option', { name: '1/4 tile' })).toBeVisible()
    expect(screen.getByRole('option', { name: '3 seconds' })).toBeVisible()
    await user.selectOptions(screen.getByLabelText('Starting speed'), '4')
    await user.selectOptions(screen.getByLabelText('Turning tolerance'), '64')
    await user.selectOptions(screen.getByLabelText('Round wins needed'), '1')
    await user.selectOptions(screen.getByLabelText('Bomb fuse'), '60')
    expect(screen.getByText('Apply these changes before readying up.')).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))

    expect(apply).toHaveBeenCalledWith({
      startingSpeedLevel: 4, turnAssistUnits: 64, winsToFinish: 1, bombFuseTicks: 60,
      enabledItems: ['heart'],
    })
  })

  it('explains editor, ready and match locks in both supported languages', () => {
    const shared = room(1)
    shared.settingsEditor = 'two'
    const view = render(<MantineProvider env="test"><GameplaySettingsPanel room={shared} youSeat="one"
      pending={false} copy={en} onApply={() => true} /></MantineProvider>)
    expect(screen.getByText('Player two can edit while unready.')).toBeVisible()
    expect(screen.getByRole('button', { name: 'Apply rules' })).toBeDisabled()

    shared.settingsEditor = 'one'
    shared.seats[0].ready = true
    view.rerender(<MantineProvider env="test"><GameplaySettingsPanel room={shared} youSeat="one"
      pending={false} copy={pl} onApply={() => true} /></MantineProvider>)
    expect(screen.getByText('Wycofaj gotowość, aby edytować zasady.')).toBeVisible()
  })

  it('restores the complete default draft before explicitly applying it', async () => {
    const apply = vi.fn(() => true)
    const user = userEvent.setup()
    const custom = room(2)
    custom.settings = {
      startingSpeedLevel: 4, turnAssistUnits: 64, winsToFinish: 1, bombFuseTicks: 60, enabledItems: [],
    }
    render(<MantineProvider env="test"><GameplaySettingsPanel room={custom} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)

    await user.click(screen.getByRole('button', { name: 'Restore match defaults' }))
    expect(apply).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenCalledWith({
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
      enabledItems: ['heart'],
    })
  })

  it('offers an availability-driven Heart toggle and a distinct Classic preset', async () => {
    const apply = vi.fn(() => true)
    const user = userEvent.setup()
    const view = render(<MantineProvider env="test"><GameplaySettingsPanel room={room(1)} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)

    expect(screen.getByRole('checkbox', { name: 'Heart' })).toBeChecked()
    expect(screen.getByText(/Absorbs one lethal blast/)).toBeVisible()
    await user.click(screen.getByRole('button', { name: 'Classic settings' }))
    expect(screen.getByRole('checkbox', { name: 'Heart' })).not.toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith({
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3,
      bombFuseTicks: 180, enabledItems: [],
    })

    const unavailable = room(2)
    unavailable.availableItems = []
    unavailable.settings.enabledItems = []
    view.rerender(<MantineProvider env="test"><GameplaySettingsPanel room={unavailable} youSeat="one"
      pending={false} copy={pl} onApply={apply} /></MantineProvider>)
    expect(screen.queryByRole('checkbox', { name: 'Serce' })).not.toBeInTheDocument()
    expect(pl.settings.heartDescription).toMatch(/śmiertelnym wybuchem/)
  })
})
