import type { ServerMessage } from './v1/schema'

export const archivedV1Welcome = {
  type: 'welcome',
  protocolVersion: 1,
  maxPlayers: 2,
} as const satisfies ServerMessage
