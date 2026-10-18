import type { SnapshotMessage } from '../src/game/arena/types'

const WIDTH = 15
const HEIGHT = 13

function terrain(): SnapshotMessage['state']['arena']['terrain'] {
  return Array.from({ length: WIDTH * HEIGHT }, (_value, index) => {
    const column = index % WIDTH
    const row = Math.floor(index / WIDTH)
    const border = column === 0 || column === WIDTH - 1 || row === 0 || row === HEIGHT - 1
    const pillar = column % 2 === 0 && row % 2 === 0
    return border || pillar ? 'wall' : 'floor'
  })
}

const fixedTerrain = terrain()

function motion(ordinal: number): { offset: number; heading: 'east' | 'west' } {
  const phase = (ordinal - 1) % 32
  return {
    offset: (phase <= 16 ? phase : 32 - phase) * 64,
    heading: phase > 0 && phase <= 16 ? 'east' : 'west',
  }
}

export function diagnosticSnapshot(
  runningUpdates: number,
  motionOrdinal: number = runningUpdates,
): SnapshotMessage {
  const actorMotion = motion(motionOrdinal)
  return {
    room: {
      roomId: 'renderer-diagnostic-room',
      roundId: 1,
      phase: { type: 'running' },
    },
    state: {
      physicalTick: runningUpdates,
      arena: {
        width: WIDTH,
        height: HEIGHT,
        terrain: fixedTerrain,
        blocks: [
          { column: 3, row: 3 }, { column: 5, row: 3 },
          { column: 9, row: 7 }, { column: 11, row: 9 },
        ],
      },
      actors: [
        {
          id: 10, seat: 'one', position: { x: 1_536 + actorMotion.offset, y: 1_536 },
          heading: actorMotion.heading, alive: true,
        },
        {
          id: 20, seat: 'two', position: { x: 12_800 + actorMotion.offset, y: 11_776 },
          heading: actorMotion.heading, alive: true,
        },
      ],
      bombs: [],
      flames: [],
      pickups: [],
    },
  } satisfies SnapshotMessage
}
