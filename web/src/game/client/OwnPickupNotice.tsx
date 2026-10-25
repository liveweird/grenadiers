import { useState } from 'react'
import type { RenderablePickupKind } from '../arena/types'
import type { GameClientState } from './clientTypes'
import { ItemIcon } from './ItemIcon'
import { format, type GameCopy } from './locale'
import { OwnPickupCursor } from './ownPickupCursor'

function pickupLabel(kind: RenderablePickupKind, copy: GameCopy): string {
  if (kind === 'bombCapacity' || kind === 'blastRange' || kind === 'movementSpeed') {
    return copy.pickups[kind]
  }
  return copy.items[kind]
}

export function OwnPickupNotice({ state, copy }: { state: GameClientState; copy: GameCopy }) {
  const [cursor] = useState(() => new OwnPickupCursor())
  const feedback = cursor.consume(state)
  return <div className="own-pickup-notice" aria-live="polite" aria-atomic="true">
    {feedback && <span key={feedback.identity}>
      <ItemIcon kind={feedback.kind} />
      {format(copy.pickups.collected, { item: pickupLabel(feedback.kind, copy) })}
    </span>}
  </div>
}
