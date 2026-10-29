import type { SnapshotMessage } from '../game/arena/types'

export function gameSnapshot(): SnapshotMessage {
  return {
    room: {
      roomId: 'a'.repeat(32), roundId: 1, phase: { type: 'running' }, presentationBaseline: 'initial',
    },
    state: {
      physicalTick: 100,
      arena: {
        width: 15, height: 13,
        terrain: Array.from({ length: 195 }, (_, index) => {
          const column = index % 15
          const row = Math.floor(index / 15)
          return column === 0 || column === 14 || row === 0 || row === 12 ? 'wall' : 'floor'
        }),
        blocks: [{ column: 4, row: 1 }],
      },
      actors: [
        { id: 10, seat: 'one', position: { x: 1536, y: 1536 }, heading: 'east', alive: true,
          lastTeleportTick: null },
        { id: 20, seat: 'two', position: { x: 13_824, y: 11_776 }, heading: 'west', alive: true,
          lastTeleportTick: null },
      ],
      bombs: [{
        id: 30, cell: { column: 1, row: 1 }, owner: 10, range: 2, placedAt: 1,
        trigger: { type: 'timed', remainingTicks: 180, paused: false }, blast: 'standard',
      }],
      flames: [{ cell: { column: 3, row: 3 }, expiresAt: 130 }],
      pickups: [],
      oilBarrels: [],
      stormCloud: null,
    },
  }
}
