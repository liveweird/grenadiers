import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import en from '../../locales/en/game.json'
import pl from '../../locales/pl/game.json'
import type { VisibleActorView } from '../v17/schema'
import { OwnStatusBar } from './OwnStatusBar'

function actor(): VisibleActorView {
  return {
    type: 'visible', id: 1, seat: 'one', position: { x: 512, y: 512 }, heading: 'east', alive: true,
    speedLevel: 4, bombCapacity: 5, bombRange: 6, liveBombCount: 2,
    inventory: { equipment: 'heart', weapon: 'powerBomb' }, damageGraceRemainingTicks: 61,
    voluntaryActionsFrozen: true, lastTeleportTick: null,
    stormSlowRemainingTicks: 0,
    effects: [
      { kind: 'shield', remainingTicks: 1 },
      { kind: 'stopwatch', remainingTicks: 60 },
      { kind: 'invisibilityCloak', remainingTicks: 61 },
      { kind: 'goldenFlame', remainingTicks: 120 },
      { kind: 'shortFuse', remainingTicks: 180 },
      { kind: 'diarrhea', remainingTicks: 240 },
      { kind: 'slowFoot', remainingTicks: 300 },
      { kind: 'slowFoot', remainingTicks: 300 },
      { kind: 'shortFlame', remainingTicks: 360 },
      { kind: 'longFuse', remainingTicks: 420 },
      { kind: 'reverseControls', remainingTicks: 480 },
      { kind: 'invisibleBombs', remainingTicks: 540 },
    ],
  }
}

describe('own status bar', () => {
  it('shows authoritative totals, slots, every effect, equal slow stacks, freeze and grace', () => {
    render(<OwnStatusBar actor={actor()} copy={en} />)
    const bar = screen.getByRole('region', { name: 'Your items' })
    expect(bar).toHaveTextContent('Bombs available3')
    expect(bar).toHaveTextContent('Bomb capacity5')
    expect(bar).toHaveTextContent('Blast range6')
    expect(bar).toHaveTextContent('Speed level5')
    expect(bar).toHaveTextContent('GearHeart')
    expect(bar).toHaveTextContent('Bomb typePower bomb')
    for (const text of [
      'Shield (0:01)', 'Stopwatch (0:01)', 'Invisibility cloak (0:02)', 'Golden flame (0:02)',
      'Short fuse (0:03)', 'Diarrhea (0:04)', 'Slow foot ×2 (1/4 speed; 0:05, 0:05)',
      'Short flame (0:06)', 'Long fuse (0:07)', 'Reverse controls (0:08)',
      'Invisible bombs (0:09)', 'Frozen by Stopwatch', 'Damage grace0:02',
    ]) expect(bar).toHaveTextContent(text)
    expect(bar.closest('[aria-live]')).toBeNull()
    expect(bar.querySelector('.item-icon')).toHaveStyle({ backgroundSize: '560px 952px' })
    expect(screen.getByLabelText('Protected from damage, 0:02 remaining')).toHaveTextContent('0:02')
  })

  it('reflects consumed and replacement slots without retaining prior state', () => {
    const current = actor()
    const view = render(<OwnStatusBar actor={current} copy={en} />)
    current.inventory = { equipment: 'wallPass', weapon: null }
    current.effects = []
    current.damageGraceRemainingTicks = 0
    current.voluntaryActionsFrozen = false
    view.rerender(<OwnStatusBar actor={{ ...current }} copy={en} />)
    const bar = screen.getByRole('region', { name: 'Your items' })
    expect(bar).toHaveTextContent('GearWall pass')
    expect(bar).toHaveTextContent('Bomb typeNone')
    expect(bar).toHaveTextContent('EffectsNone')
    expect(bar).not.toHaveTextContent('Power bomb')
    expect(bar).not.toHaveTextContent('Shield (')
  })

  it('includes the authoritative grace time in the localized accessible value', () => {
    render(<OwnStatusBar actor={actor()} copy={pl} />)
    expect(screen.getByLabelText('Ochrona przed obrażeniami aktywna, pozostało 0:02'))
      .toHaveTextContent('0:02')
  })

  it('shows the own cloud slowdown separately and clears it at zero', () => {
    const current = { ...actor(), effects: [], stormSlowRemainingTicks: 180 }
    const view = render(<OwnStatusBar actor={current} copy={en} />)
    expect(screen.getByText('Storm cloud')).toBeVisible()
    expect(screen.getByLabelText('Storm cloud slowdown, 0:03 remaining'))
      .toHaveTextContent('0:03 · 50% slowdown')

    view.rerender(<OwnStatusBar actor={{ ...current, stormSlowRemainingTicks: 0 }} copy={en} />)
    expect(screen.queryByText('Storm cloud')).not.toBeInTheDocument()
  })
})
