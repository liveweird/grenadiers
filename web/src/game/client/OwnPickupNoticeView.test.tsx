import { render, screen } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import en from '../../locales/en/game.json'
import pl from '../../locales/pl/game.json'
import type { GameClientState } from './clientTypes'
import { snapshot } from './testSupport'
import { OwnPickupNotice } from './OwnPickupNotice'

function admittedState(): GameClientState {
  const current = snapshot(20)
  current.state.physicalTick = 20
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one',
    failure: null, admissionPending: false, leavePending: false, settingsPending: false,
    reconnectAttempt: 0,
    admissionBaseline: {
      roomId: current.room.roomId, revision: 20, tick: 20, physicalTick: 20,
    },
  }
}

describe('own pickup notice', () => {
  it('announces one recipient event with a sprite and keeps repeated frames mounted', () => {
    const current = admittedState()
    const view = render(<OwnPickupNotice state={current} copy={en} />)
    const live = document.querySelector('.own-pickup-notice')
    expect(live).toHaveAttribute('aria-live', 'polite')
    expect(live).toBeEmptyDOMElement()

    current.room!.revision = 21
    current.snapshot!.room.revision = 21
    current.snapshot!.state.physicalTick = 21
    current.snapshot!.events = [{
      tick: 21, index: 0,
      kind: {
        type: 'pickupCollected', id: 1, seat: 'one',
        cell: { column: 2, row: 2 }, kind: 'teleporter',
      },
    }]
    view.rerender(<OwnPickupNotice state={{ ...current }} copy={en} />)
    const announcement = screen.getByText('Picked up Teleporter.')
    expect(announcement.querySelector('.item-icon')).toBeInTheDocument()
    view.rerender(<OwnPickupNotice state={{ ...current }} copy={en} />)
    expect(screen.getByText('Picked up Teleporter.')).toBe(announcement)

    view.rerender(<OwnPickupNotice state={{ ...current }} copy={pl} />)
    expect(screen.getByText('Zdobyto: Teleporter.')).toBe(announcement)
  })
})
