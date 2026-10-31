import { MantineProvider } from '@mantine/core'
import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { StrictMode } from 'react'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import i18n from '../../i18n'
import type { GameClientState } from '../client/client'
import { useGameCopy } from '../client/locale'
import { snapshot } from '../client/testSupport'
import { SoundControl } from './SoundControl'

function state(): GameClientState {
  const current = snapshot(20)
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: { roomId: current.room.roomId, revision: 20, tick: 20, physicalTick: 20 },
  }
}

function Subject({ current = state() }: { current?: GameClientState }) {
  return <SoundControl state={current} copy={useGameCopy()} />
}

function view(element = <Subject />) {
  return render(<MantineProvider env="test">{element}</MantineProvider>)
}

function audioContext() {
  const oscillator = vi.fn(() => ({
    type: 'sine', frequency: { setValueAtTime: vi.fn(), exponentialRampToValueAtTime: vi.fn() },
    connect: vi.fn(), disconnect: vi.fn(), start: vi.fn(), stop: vi.fn(), onended: null,
  }))
  const raw = {
    state: 'suspended', currentTime: 1, destination: {},
    resume: vi.fn(async () => { raw.state = 'running' }),
    suspend: vi.fn(async () => { raw.state = 'suspended' }),
    close: vi.fn(async () => { raw.state = 'closed' }),
    createGain: vi.fn(() => ({
      gain: { setValueAtTime: vi.fn(), cancelAndHoldAtTime: vi.fn(), linearRampToValueAtTime: vi.fn(),
        exponentialRampToValueAtTime: vi.fn() },
      connect: vi.fn(), disconnect: vi.fn(),
    })),
    createOscillator: oscillator,
  }
  return { raw: raw as unknown as AudioContext, internal: raw, oscillator }
}

describe('sound control gesture and localization', () => {
  beforeEach(async () => { localStorage.clear(); await i18n.changeLanguage('en') })
  afterEach(() => vi.unstubAllGlobals())

  it('constructs no context on mount, restored preference, or StrictMode remount', () => {
    localStorage.setItem('grenadiers.audio.v1', '{"version":1,"enabled":true}')
    const Context = vi.fn()
    vi.stubGlobal('AudioContext', Context)
    view(<StrictMode><Subject /></StrictMode>)
    expect(Context).not.toHaveBeenCalled()
    expect(screen.getByRole('button', { name: 'Resume sound' })).toBeVisible()
    expect(screen.getByText('Sound paused')).toBeVisible()
  })

  it('enables and plays after the StrictMode effect rehearsal', async () => {
    localStorage.setItem('grenadiers.audio.v1', '{"version":1,"enabled":true}')
    const fake = audioContext()
    const Context = vi.fn(function AudioContextStub() { return fake.raw })
    vi.stubGlobal('AudioContext', Context)
    const initial = state()
    const rendered = view(<StrictMode><Subject current={initial} /></StrictMode>)
    await userEvent.setup().click(screen.getByRole('button', { name: 'Resume sound' }))
    await vi.waitFor(() => expect(screen.getByText('Sound on')).toBeVisible())
    expect(Context).toHaveBeenCalledOnce()

    const later = state()
    later.room!.revision = 21
    later.room!.tick = 21
    later.snapshot!.room.revision = 21
    later.snapshot!.room.tick = 21
    later.snapshot!.state.physicalTick = 21
    later.snapshot!.events = [{
      tick: 21, index: 0,
      kind: { type: 'bombPlaced', bomb: {} as never },
    }]
    later.admissionBaseline = initial.admissionBaseline
    rendered.rerender(<MantineProvider env="test"><StrictMode>
      <Subject current={later} />
    </StrictMode></MantineProvider>)
    await vi.waitFor(() => expect(fake.oscillator).toHaveBeenCalledOnce())

    const oil = state()
    oil.room!.revision = 22
    oil.snapshot!.room.revision = 22
    oil.snapshot!.state.physicalTick = 22
    oil.snapshot!.room.settings.enabledItems = ['oilBarrel']
    oil.snapshot!.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'unlit' }]
    oil.admissionBaseline = initial.admissionBaseline
    rendered.rerender(<MantineProvider env="test"><StrictMode>
      <Subject current={oil} />
    </StrictMode></MantineProvider>)
    await vi.waitFor(() => expect(screen.getByText('Sound on')).toBeVisible())
    expect(fake.oscillator).toHaveBeenCalledOnce()

    const ignited = state()
    ignited.room!.revision = 23
    ignited.snapshot!.room.revision = 23
    ignited.snapshot!.state.physicalTick = 23
    ignited.snapshot!.room.settings.enabledItems = ['oilBarrel']
    ignited.snapshot!.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'burning' }]
    ignited.admissionBaseline = initial.admissionBaseline
    rendered.rerender(<MantineProvider env="test"><StrictMode>
      <Subject current={ignited} />
    </StrictMode></MantineProvider>)
    await vi.waitFor(() => expect(fake.oscillator).toHaveBeenCalledOnce())

    const resumed = state()
    resumed.room!.revision = 30
    resumed.snapshot!.room.revision = 30
    resumed.snapshot!.state.physicalTick = 30
    resumed.snapshot!.room.settings.enabledItems = ['oilBarrel']
    resumed.snapshot!.state.oilBarrels = [{ cell: { column: 3, row: 1 }, state: 'unlit' }]
    resumed.admissionBaseline = {
      roomId: resumed.room!.roomId, revision: 30, tick: 30, physicalTick: 30,
    }
    rendered.rerender(<MantineProvider env="test"><StrictMode>
      <Subject current={resumed} />
    </StrictMode></MantineProvider>)
    await vi.waitFor(() => expect(screen.getByText('Sound on')).toBeVisible())
    expect(fake.oscillator).toHaveBeenCalledOnce()
  })

  it('creates from the native button gesture and reports local failure', async () => {
    const Context = vi.fn(() => { throw new Error('unsupported') })
    vi.stubGlobal('AudioContext', Context)
    view()
    expect(screen.getByRole('button', { name: 'Enable sound' })).toBeVisible()
    fireEvent.click(screen.getByRole('button', { name: 'Enable sound' }))
    await vi.waitFor(() => expect(screen.getByText('Sound unavailable')).toBeVisible())
    expect(Context).toHaveBeenCalledOnce()
  })

  it('exposes Polish button and status copy', async () => {
    await i18n.changeLanguage('pl')
    view()
    expect(screen.getByRole('button', { name: 'Włącz dźwięk' })).toBeVisible()
    expect(screen.getByText('Dźwięk wyłączony')).toBeVisible()
    expect(screen.getByRole('slider', { name: 'Głośność dźwięku' })).toHaveValue('60')
  })

  it('restores, bounds, and persists the accessible volume without creating a context', () => {
    localStorage.setItem('grenadiers.audio.v1', '{"version":2,"enabled":false,"volume":35}')
    const Context = vi.fn()
    vi.stubGlobal('AudioContext', Context)
    view()
    const slider = screen.getByRole('slider', { name: 'Sound volume' })
    expect(slider).toHaveValue('35')
    fireEvent.change(slider, { target: { value: '85' } })
    expect(slider).toHaveValue('85')
    expect(localStorage.getItem('grenadiers.audio.v1'))
      .toBe('{"version":2,"enabled":false,"volume":85}')
    expect(Context).not.toHaveBeenCalled()
  })

  it('pauses for pagehide, resumes from a later gesture, and removes the lifecycle listener', async () => {
    localStorage.setItem('grenadiers.audio.v1', '{"version":1,"enabled":true}')
    const fake = audioContext()
    const Context = vi.fn(function AudioContextStub() { return fake.raw })
    vi.stubGlobal('AudioContext', Context)
    const rendered = view()
    await userEvent.setup().click(screen.getByRole('button', { name: 'Resume sound' }))
    await vi.waitFor(() => expect(screen.getByText('Sound on')).toBeVisible())
    window.dispatchEvent(new Event('pagehide'))
    await vi.waitFor(() => expect(screen.getByText('Sound paused')).toBeVisible())
    window.dispatchEvent(new Event('pageshow'))
    await userEvent.setup().click(screen.getByRole('button', { name: 'Resume sound' }))
    await vi.waitFor(() => expect(screen.getByText('Sound on')).toBeVisible())
    expect(Context).toHaveBeenCalledOnce()
    expect(fake.internal.resume).toHaveBeenCalledTimes(2)
    expect(localStorage.getItem('grenadiers.audio.v1')).toContain('"enabled":true')

    rendered.unmount()
    const suspends = fake.internal.suspend.mock.calls.length
    window.dispatchEvent(new Event('pagehide'))
    expect(fake.internal.suspend).toHaveBeenCalledTimes(suspends)
  })
})
