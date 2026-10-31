import { snapshot as currentSnapshot } from '../client/testSupport'
import type { Snapshot } from './schema'

export function snapshot(revision: number): Snapshot {
  const value = structuredClone(currentSnapshot(revision)) as unknown as {
    room: { rulesVersion: number; configuration: { environment: Record<string, unknown> };
      settings: { environment: Record<string, unknown> } }
    state: Record<string, unknown>
  }
  value.room.rulesVersion = 9
  delete value.room.configuration.environment.spikePits
  delete value.room.settings.environment.spikePits
  delete value.state.spikePits
  return value as unknown as Snapshot
}
