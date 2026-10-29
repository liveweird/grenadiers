import { MantineProvider } from '@mantine/core'
import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import en from '../../locales/en/game.json'
import pl from '../../locales/pl/game.json'
import { GameplaySettingsPanel } from './GameplaySettingsPanel'
import { room } from './testSupport'

function panel(current: ReturnType<typeof room>, copy: typeof en) {
  return <MantineProvider env="test"><GameplaySettingsPanel key={current.settingsRevision}
    room={current} youSeat="one" pending={false} copy={copy} onApply={() => true} />
  </MantineProvider>
}

describe('v16 lair settings controls', () => {
  it('exposes accessible Polish names for the toggle and both numeric controls', () => {
    render(panel(room(1), pl))
    expect(screen.getByRole('checkbox', { name: 'Legowiska potworów' })).toBeChecked()
    expect(screen.getByRole('textbox', { name: 'Czas odrodzenia (sekundy)' })).toHaveValue('10')
    expect(screen.getByRole('textbox', { name: 'Żywe potwory na legowisko' })).toHaveValue('3')
  })

  it('remounts a stale draft from an accepted higher settings revision', () => {
    const current = room(1)
    const view = render(panel(current, en))
    const spawn = screen.getByRole('textbox', { name: 'Spawn interval (seconds)' })
    fireEvent.change(spawn, { target: { value: '24' } })
    expect(spawn).toHaveValue('24')

    const accepted = room(2)
    accepted.settingsRevision = 2
    accepted.settings.environment.lairs.spawnSeconds = 30
    accepted.configuration.environment.lairs.spawnSeconds = 30
    view.rerender(panel(accepted, en))
    expect(screen.getByRole('textbox', { name: 'Spawn interval (seconds)' })).toHaveValue('30')
  })
})
