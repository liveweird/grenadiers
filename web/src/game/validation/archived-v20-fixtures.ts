import { snapshot as currentSnapshot } from '../client/testSupport'

type SettingsFrame = { room: { settings: { bombCapacityDropMultiplier?: unknown } } }

/** Clone a current fixture while removing fields unknown to sealed v20-and-earlier schemas. */
export function archivedFrame<T>(value: T): T {
  const archived = structuredClone(value) as T & SettingsFrame
  delete archived.room.settings.bombCapacityDropMultiplier
  return archived
}

export function snapshot(revision: number, roundId = 1) {
  return archivedFrame(currentSnapshot(revision, roundId))
}
