/** One world unit is one tile; authoritative positions contain 1024 simulation units per tile. */
export const WORLD_TILE = 1

export function cellToWorld(column: number, row: number): [number, number] {
  return [(column + 0.5) * WORLD_TILE, (row + 0.5) * WORLD_TILE]
}

export function simulationToWorld(x: number, y: number): [number, number] {
  return [x / 1024, y / 1024]
}
