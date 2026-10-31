import { fireEvent, render, screen } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import i18n from '../../i18n'
import { CpuDebugPanel } from './CpuDebugPanel'
import type { GameClientState } from './clientTypes'
import type { CpuDebugResponse } from './cpuDebugTypes'
import { snapshot } from './testSupport'

function state(): GameClientState {
  const current = snapshot(20)
  current.room.configuration.controllers[1] = 'cpu'
  current.room.configuration.cpuDifficulty = 'standard'
  current.room.seats[1].controller = { type: 'cpu', difficulty: 'standard' }
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: { roomId: current.room.roomId, revision: 20, tick: 20, physicalTick: 20 },
  }
}

const response: CpuDebugResponse = {
  protocolVersion: 1, roomId: 'b'.repeat(32), roundId: 1, tick: 20, revision: 20, youSeat: 'one',
  cpus: [{
    seat: 'two', status: 'available', observedTick: 20, objective: 'attack',
    target: { type: 'opponent', seat: 'one' }, approach: { column: 2, row: 3 },
    phase: 'approach', switchReason: 'betterOpportunity',
  }],
}

beforeEach(async () => { await i18n.changeLanguage('en') })

describe('CPU intent developer panel', () => {
  it('is a compact native disclosure with visible localized intent fields', () => {
    render(<CpuDebugPanel state={state()} debug={{ type: 'ready', response }} />)
    const region = screen.getByRole('region', { name: 'CPU intent diagnostics' })
    const disclosure = screen.getByRole('button', { name: 'CPU intent diagnostics' })
    expect(disclosure).toHaveAccessibleName('CPU intent diagnostics')
    expect(disclosure).toHaveAttribute('aria-expanded', 'false')
    fireEvent.click(disclosure)
    expect(disclosure).toHaveAttribute('aria-expanded', 'true')
    expect(region).toHaveTextContent('ObjectiveAttack')
    expect(region).toHaveTextContent('TargetPlayer one')
    expect(region).toHaveTextContent('PhaseApproach')
    expect(region).toHaveTextContent('Latest switch reasonBetter opportunity')
    expect(region).toHaveTextContent('Observation tick20')
  })

  it('provides the full diagnostic interface in Polish', async () => {
    await i18n.changeLanguage('pl')
    render(<CpuDebugPanel state={state()} debug={{ type: 'ready', response }} />)
    const region = screen.getByRole('region', { name: 'Diagnostyka zamiarów komputera' })
    const disclosure = screen.getByRole('button', { name: 'Diagnostyka zamiarów komputera' })
    expect(disclosure).toHaveAccessibleName('Diagnostyka zamiarów komputera')
    fireEvent.click(disclosure)
    expect(region).toHaveTextContent('CelAtak')
    expect(region).toHaveTextContent('ObiektGracz pierwszy')
    expect(region).toHaveTextContent('FazaPodejście')
    expect(region).toHaveTextContent('Ostatni powód zmianyLepsza okazja')
  })

  it('removes every detail immediately when the latest public snapshot hides the CPU', () => {
    const current = state()
    const view = render(<CpuDebugPanel state={current} debug={{ type: 'ready', response }} />)
    fireEvent.click(screen.getByRole('button', { name: 'CPU intent diagnostics' }))
    expect(screen.getByRole('region', { name: 'CPU intent diagnostics' })).toHaveTextContent('Attack')
    current.snapshot!.state.actors[1] = { type: 'hidden', id: 2, seat: 'two', alive: true }
    view.rerender(<CpuDebugPanel state={{ ...current }} debug={{ type: 'ready', response }} />)
    const region = screen.getByRole('region', { name: 'CPU intent diagnostics' })
    expect(region).toHaveTextContent('Intent unavailable for the current view.')
    expect(region).not.toHaveTextContent('Attack')
    expect(region).not.toHaveTextContent('Better opportunity')
  })
})
