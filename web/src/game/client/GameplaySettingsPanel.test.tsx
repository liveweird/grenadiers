import { MantineProvider } from '@mantine/core'
import { fireEvent, render, screen } from '@testing-library/react'
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
      ...room(1).settings,
      startingSpeedLevel: 4, turnAssistUnits: 64, winsToFinish: 1, bombFuseTicks: 60,
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
      environment: { stormCloud: false, lairs: { enabled: false, spawnSeconds: 24, maxLivingMobs: 1 } },
    }
    render(<MantineProvider env="test"><GameplaySettingsPanel room={custom} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)

    await user.click(screen.getByRole('button', { name: 'Restore match defaults' }))
    expect(apply).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenCalledWith({
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3, bombFuseTicks: 180,
      enabledItems: custom.availableItems,
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 10, maxLivingMobs: 1 } },
    })
  })
})

describe('gameplay settings items', () => {
  it('toggles all optional items independently and keeps defaults distinct from Classic', async () => {
    const apply = vi.fn(() => true)
    const user = userEvent.setup()
    const current = room(1)
    current.settings.enabledItems = ['heart']
    render(<MantineProvider env="test"><GameplaySettingsPanel room={current} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)

    expect(screen.getByRole('checkbox', { name: 'Kick' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Bracelet' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Wall pass' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Heart' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Storm cloud' })).toBeChecked()
    expect(screen.getByText(/roaming cloud slows players by 50%/)).toBeVisible()
    expect(screen.getByRole('checkbox', { name: 'Remote bomb' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Piercing bomb' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Power bomb' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Shield' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Golden flame' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short fuse' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Slow foot' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short flame' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Long fuse' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Reverse controls' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Invisible bombs' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Oil barrel' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Teleporter' })).not.toBeChecked()
    expect(screen.getByText(/move through breakable blocks/)).toBeVisible()
    expect(screen.getByText(/Hard walls and bombs still block/)).toBeVisible()
    expect(screen.getByText(/leave a block you already overlap but cannot re-enter/)).toBeVisible()
    expect(screen.getByText(/Absorbs one lethal blast/)).toBeVisible()
    expect(screen.getByText(/remotely triggered bomb/)).toBeVisible()
    expect(screen.getByText(/kicks a stationary timed or remote bomb/)).toBeVisible()
    expect(screen.getByText(/blast passes through breakable blocks/)).toBeVisible()
    expect(screen.getByText(/diamond-shaped area/)).toBeVisible()
    expect(screen.getByText(/entering it later detonates it/)).toBeVisible()
    expect(screen.getByText(/5 seconds of damage immunity/)).toBeVisible()
    expect(screen.getByText(/Hides your actor and attributable cues/)).toBeVisible()
    expect(screen.getByText(/Maximizes new bomb blasts for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Halves the fuse of new bombs for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Automatically drops ordinary bombs for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Halves movement speed for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Limits new bomb blasts to 1 tile for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Doubles the fuse of new bombs for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Reverses movement controls for 10 seconds/)).toBeVisible()
    expect(screen.getByText(/Hides every bomb and its placement or detonation cue/)).toBeVisible()
    expect(screen.getByText(/unsafe pickup is still consumed/)).toBeVisible()
    expect(screen.getByText(/up to 8 Oil Barrel objects/)).toBeVisible()
    expect(screen.getByText(/Wall Pass can cross an unlit barrel but does not protect/)).toBeVisible()
    await user.click(screen.getByRole('checkbox', { name: 'Kick' }))
    expect(screen.getByText(/Press your Equipment key to punch/)).toBeVisible()
    await user.click(screen.getByRole('checkbox', { name: 'Punch' }))
    expect(screen.getByText(/shoves them up to 1 tile/)).toBeVisible()
    await user.click(screen.getByRole('checkbox', { name: 'Bracelet' }))
    await user.click(screen.getByRole('checkbox', { name: 'Wall pass' }))
    await user.click(screen.getByRole('checkbox', { name: 'Remote bomb' }))
    await user.click(screen.getByRole('checkbox', { name: 'Piercing bomb' }))
    await user.click(screen.getByRole('checkbox', { name: 'Power bomb' }))
    await user.click(screen.getByRole('checkbox', { name: 'Landmine' }))
    await user.click(screen.getByRole('checkbox', { name: 'Shield' }))
    await user.click(screen.getByRole('checkbox', { name: 'Stopwatch' }))
    await user.click(screen.getByRole('checkbox', { name: 'Invisibility cloak' }))
    await user.click(screen.getByRole('checkbox', { name: 'Golden flame' }))
    await user.click(screen.getByRole('checkbox', { name: 'Short fuse' }))
    await user.click(screen.getByRole('checkbox', { name: 'Diarrhea' }))
    await user.click(screen.getByRole('checkbox', { name: 'Slow foot' }))
    await user.click(screen.getByRole('checkbox', { name: 'Short flame' }))
    await user.click(screen.getByRole('checkbox', { name: 'Long fuse' }))
    await user.click(screen.getByRole('checkbox', { name: 'Reverse controls' }))
    await user.click(screen.getByRole('checkbox', { name: 'Invisible bombs' }))
    await user.click(screen.getByRole('checkbox', { name: 'Oil barrel' }))
    await user.click(screen.getByRole('checkbox', { name: 'Teleporter' }))
    expect(screen.getByRole('checkbox', { name: 'Heart' })).toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith(expect.objectContaining({
      enabledItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    }))
    await user.click(screen.getByRole('checkbox', { name: 'Golden flame' }))
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith(expect.objectContaining({
      enabledItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    }))
    await user.click(screen.getByRole('checkbox', { name: 'Golden flame' }))
    await user.click(screen.getByRole('checkbox', { name: 'Heart' }))
    expect(screen.getByRole('checkbox', { name: 'Shield' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Stopwatch' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Piercing bomb' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Power bomb' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Landmine' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Invisibility cloak' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Golden flame' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short fuse' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Diarrhea' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Slow foot' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short flame' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Storm cloud' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Long fuse' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Reverse controls' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Oil barrel' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Teleporter' })).toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith(expect.objectContaining({
      enabledItems: ['kick', 'punch', 'bracelet', 'wallPass', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    }))
    await user.click(screen.getByRole('checkbox', { name: 'Shield' }))
    expect(screen.getByRole('checkbox', { name: 'Golden flame' })).toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith(expect.objectContaining({
      enabledItems: ['kick', 'punch', 'bracelet', 'wallPass', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    }))
    await user.click(screen.getByRole('checkbox', { name: 'Heart' }))
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith(expect.objectContaining({
      enabledItems: ['kick', 'punch', 'bracelet', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse', 'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'oilBarrel', 'teleporter'],
    }))
    await user.click(screen.getByRole('button', { name: 'Classic settings' }))
    expect(screen.getByRole('checkbox', { name: 'Kick' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Punch' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Bracelet' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Wall pass' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Heart' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Remote bomb' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Piercing bomb' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Power bomb' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Shield' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Stopwatch' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Invisibility cloak' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Golden flame' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short fuse' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Diarrhea' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Slow foot' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short flame' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Long fuse' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Reverse controls' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Oil barrel' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Teleporter' })).not.toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Storm cloud' })).not.toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith({
      startingSpeedLevel: 0, turnAssistUnits: 256, winsToFinish: 3,
      bombFuseTicks: 180, enabledItems: [],
      environment: { stormCloud: false, lairs: { enabled: false, spawnSeconds: 10, maxLivingMobs: 1 } },
    })
    await user.click(screen.getByRole('checkbox', { name: 'Shield' }))
    await user.click(screen.getByRole('checkbox', { name: 'Short flame' }))
    await user.click(screen.getByRole('button', { name: 'Restore match defaults' }))
    expect(screen.getByRole('checkbox', { name: 'Heart' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Shield' })).toBeChecked()
    expect(screen.getByRole('checkbox', { name: 'Short flame' })).toBeChecked()
  })
})

describe('gameplay settings bulk actions', () => {
  it('retains lair numeric choices while bulk actions toggle the environment', async () => {
    const apply = vi.fn(() => true)
    const user = userEvent.setup()
    render(<MantineProvider env="test"><GameplaySettingsPanel room={room(1)} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)
    const seconds = screen.getByRole('textbox', { name: 'Respawn delay (seconds)' })
    expect(screen.queryByRole('textbox', { name: 'Living monsters per lair' })).not.toBeInTheDocument()
    fireEvent.change(seconds, { target: { value: '4' } })
    expect(screen.getByRole('button', { name: 'Apply rules' })).toBeDisabled()
    fireEvent.change(seconds, { target: { value: '24' } })
    await user.click(screen.getByRole('button', { name: 'All off' }))
    await user.click(screen.getByRole('button', { name: 'All on' }))
    expect(seconds).toHaveValue('24')
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenCalledWith(expect.objectContaining({ environment: {
      stormCloud: true, lairs: { enabled: true, spawnSeconds: 24, maxLivingMobs: 1 },
    } }))
  })

  it('selects only canonical available items without changing numeric drafts or applying', async () => {
    const apply = vi.fn(() => true)
    const user = userEvent.setup()
    const partial = room(4)
    partial.availableItems = ['teleporter', 'heart', 'shield']
    partial.settings = {
      startingSpeedLevel: 3, turnAssistUnits: 128, winsToFinish: 7,
      bombFuseTicks: 270, enabledItems: ['heart'],
      environment: { stormCloud: false, lairs: { enabled: false, spawnSeconds: 24, maxLivingMobs: 1 } },
    }
    render(<MantineProvider env="test"><GameplaySettingsPanel room={partial} youSeat="one"
      pending={false} copy={en} onApply={apply} /></MantineProvider>)
    await user.click(screen.getByRole('button', { name: 'All on' }))
    expect(apply).not.toHaveBeenCalled()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenCalledWith({
      startingSpeedLevel: 3, turnAssistUnits: 128, winsToFinish: 7, bombFuseTicks: 270,
      enabledItems: ['heart', 'shield', 'teleporter'],
      environment: { stormCloud: true, lairs: { enabled: true, spawnSeconds: 24, maxLivingMobs: 1 } },
    })
    await user.click(screen.getByRole('button', { name: 'All off' }))
    expect(screen.getByRole('checkbox', { name: 'Heart' })).not.toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Apply rules' }))
    expect(apply).toHaveBeenLastCalledWith(expect.objectContaining({
      startingSpeedLevel: 3, turnAssistUnits: 128, winsToFinish: 7,
      bombFuseTicks: 270, enabledItems: [],
      environment: { stormCloud: false, lairs: { enabled: false, spawnSeconds: 24, maxLivingMobs: 1 } },
    }))
  })

  it('keeps empty catalogs safe and provides the agreed Polish bulk actions', async () => {
    const user = userEvent.setup()
    const empty = room(5)
    empty.availableItems = []
    empty.settings.enabledItems = []
    render(<MantineProvider env="test"><GameplaySettingsPanel room={empty} youSeat="one"
      pending={false} copy={pl} onApply={() => true} /></MantineProvider>)
    await user.click(screen.getByRole('button', { name: 'Włącz wszystkie' }))
    expect(screen.getByRole('checkbox', { name: 'Chmura burzowa' })).toBeChecked()
    await user.click(screen.getByRole('button', { name: 'Wyłącz wszystkie' }))
    expect(screen.getByRole('checkbox', { name: 'Chmura burzowa' })).not.toBeChecked()
    expect(screen.getByRole('button', { name: 'Zastosuj zasady' })).toBeEnabled()
  })
})

describe('gameplay settings Oil locale', () => {
  it('hides unavailable Oil and explains its complete Polish rules', () => {
    const unavailable = room(2)
    unavailable.availableItems = []
    unavailable.settings.enabledItems = []
    render(<MantineProvider env="test"><GameplaySettingsPanel room={unavailable} youSeat="one"
      pending={false} copy={pl} onApply={() => true} /></MantineProvider>)
    expect(screen.queryByRole('checkbox', { name: 'Serce' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Przejście przez ściany' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Tarcza' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Złoty płomień' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Krótki lont' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Biegunka' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Wolna stopa' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Krótki płomień' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Długi lont' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Odwrócone sterowanie' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Beczka oleju' })).not.toBeInTheDocument()
    expect(screen.queryByRole('checkbox', { name: 'Teleporter' })).not.toBeInTheDocument()
    expect(pl.settings.heartDescription).toMatch(/śmiertelnym wybuchem/)
    expect(pl.settings.wallPassDescription).toMatch(/zniszczalne bloki/)
    expect(pl.settings.wallPassDescription).toMatch(/Twarde ściany i bomby nadal blokują/)
    expect(pl.settings.wallPassDescription).toMatch(/nie możesz wejść do niego ponownie/)
    expect(pl.settings.kickDescription).toMatch(/co 10 aktualizacji/)
    expect(pl.settings.kickDescription).toMatch(/Min nie można kopać/)
    expect(pl.settings.braceletDescription).toMatch(/1 pole/)
    expect(pl.settings.braceletDescription).toMatch(/0,5 sekundy/)
    expect(pl.settings.braceletDescription).toMatch(/rozdzielenia graczy/)
    expect(pl.settings.shieldDescription).toMatch(/5 sekund/)
    expect(pl.settings.goldenFlameDescription).toMatch(/10 sekund/)
    expect(pl.settings.shortFuseDescription).toMatch(/10 sekund/)
    expect(pl.settings.shortFuseDescription).toMatch(/nowych bomb/)
    expect(pl.settings.diarrheaDescription).toMatch(/10 sekund/)
    expect(pl.settings.diarrheaDescription).toMatch(/miejsce na kolejną/)
    expect(pl.settings.slowFootDescription).toMatch(/6 razy/)
    expect(pl.settings.shortFlameDescription).toMatch(/1 pola/)
    expect(pl.settings.shortFlameDescription).toMatch(/Złoty płomień/)
    expect(pl.settings.longFuseDescription).toMatch(/10 sekund/)
    expect(pl.settings.longFuseDescription).toMatch(/znoszą się/)
    expect(pl.settings.reverseControlsDescription).toMatch(/10 sekund/)
    expect(pl.settings.oilBarrelDescription).toMatch(/do 8 obiektów/)
    expect(pl.settings.oilBarrelDescription).toMatch(/do końca rundy/)
    expect(pl.settings.oilBarrelDescription).toMatch(/nie chroni/)
    expect(pl.settings.teleporterDescription).toMatch(/niebezpieczny przedmiot także znika/)
  })
})

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
