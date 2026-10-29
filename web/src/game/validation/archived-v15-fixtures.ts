import { snapshot as v14Snapshot } from '../v14/testSupport'
import type { Snapshot } from '../v15/schema'

export function snapshot(revision: number, stormCloud = true): Snapshot {
  const message = structuredClone(v14Snapshot(revision)) as unknown as Snapshot
  message.room.rulesVersion = 7
  message.room.configuration.environment = { stormCloud }
  message.room.settings.environment = { stormCloud }
  message.state.physicalTick = message.room.tick
  message.state.stormCloud = stormCloud ? {
    position: { x: 7_680, y: 6_656 }, direction: 'east', legRemainingTicks: 128,
  } : null
  for (const actor of message.state.actors) {
    if (actor.type === 'visible') actor.stormSlowRemainingTicks = 0
  }
  return message
}
