import type { SnapshotMessage } from '../types'

const MAX_CELLS = 195
const MAX_LIGHTS = 4
const LIGHT_RADIUS = 2.45
const COLORS = [0xffa044, 0xffc064] as const

interface FireLightAnchor {
  /** Stable row-major identity of the represented public fire cell. */
  readonly id: string
  readonly position: readonly [number, number, number]
  readonly color: number
  readonly intensity: number
  readonly distance: number
}

export interface FireLightingModel {
  readonly anchors: readonly FireLightAnchor[]
  /** Saturating local warmth in [0, 1] at a ground-plane world position. */
  sample(x: number, z: number): number
}

function validIndex(column: number, row: number, width: number, height: number): number {
  if (!Number.isInteger(column) || !Number.isInteger(row)
    || column < 0 || row < 0 || column >= width || row >= height) return -1
  return row * width + column
}

function publicFire(snapshot: SnapshotMessage, area: number): { mask: Uint8Array; cells: number[] } {
  const { width, height } = snapshot.state.arena
  const mask = new Uint8Array(area)
  const tick = snapshot.state.physicalTick ?? -1
  for (let row = 0; row < Math.min(snapshot.state.flames.length, area); row += 1) {
    const flame = snapshot.state.flames[row]
    const index = validIndex(flame.cell.column, flame.cell.row, width, height)
    if (index >= 0 && index < area && flame.expiresAt > tick) mask[index] = 1
  }
  const oilBarrels = snapshot.state.oilBarrels ?? []
  for (let row = 0; row < Math.min(oilBarrels.length, area); row += 1) {
    const oil = oilBarrels[row]
    const index = validIndex(oil.cell.column, oil.cell.row, width, height)
    if (oil.state === 'burning' && index >= 0 && index < area) mask[index] = 1
  }
  const cells: number[] = []
  for (let index = 0; index < area; index += 1) if (mask[index]) cells.push(index)
  return { mask, cells }
}

function blockers(snapshot: SnapshotMessage, area: number): Uint8Array {
  const { arena } = snapshot.state
  const blocked = new Uint8Array(area)
  for (let index = 0; index < area; index += 1) {
    if (arena.terrain[index] === 'wall') blocked[index] = 1
  }
  const add = (cell: { column: number; row: number }) => {
    const index = validIndex(cell.column, cell.row, arena.width, arena.height)
    if (index >= 0 && index < area) blocked[index] = 1
  }
  for (let row = 0; row < Math.min(arena.blocks.length, area); row += 1) add(arena.blocks[row])
  for (let row = 0; row < Math.min(snapshot.state.lairs.length, area); row += 1) {
    add(snapshot.state.lairs[row].cell)
  }
  const oilBarrels = snapshot.state.oilBarrels ?? []
  for (let row = 0; row < Math.min(oilBarrels.length, area); row += 1) {
    if (oilBarrels[row].state === 'unlit') add(oilBarrels[row].cell)
  }
  return blocked
}

function components(mask: Uint8Array, cells: readonly number[], width: number): number[][] {
  const seen = new Uint8Array(mask.length)
  const result: number[][] = []
  for (const origin of cells) {
    if (seen[origin]) continue
    const component: number[] = []
    const queue = [origin]
    seen[origin] = 1
    for (let cursor = 0; cursor < queue.length; cursor += 1) {
      const index = queue[cursor]
      component.push(index)
      const column = index % width
      const neighbors = [index - width, index + width]
      if (column > 0) neighbors.push(index - 1)
      if (column + 1 < width) neighbors.push(index + 1)
      for (const next of neighbors) {
        if (next >= 0 && next < mask.length && mask[next] && !seen[next]) {
          seen[next] = 1
          queue.push(next)
        }
      }
    }
    component.sort((left, right) => left - right)
    result.push(component)
  }
  return result
}

function anchorCounts(groups: readonly number[][]): Array<{ cells: number[]; count: number }> {
  const selected = [...groups]
    .sort((left, right) => right.length - left.length || left[0] - right[0])
    .slice(0, MAX_LIGHTS).map((cells) => ({ cells, count: 1 }))
  for (let total = selected.length; total < MAX_LIGHTS; total += 1) {
    let choice = -1
    for (let index = 0; index < selected.length; index += 1) {
      if (selected[index].count >= selected[index].cells.length) continue
      if (choice < 0 || selected[index].cells.length / selected[index].count
        > selected[choice].cells.length / selected[choice].count) choice = index
    }
    if (choice < 0) break
    selected[choice].count += 1
  }
  return selected
}

function representatives(cells: readonly number[], count: number, width: number): number[] {
  let sumColumn = 0
  let sumRow = 0
  for (const cell of cells) { sumColumn += cell % width; sumRow += Math.floor(cell / width) }
  let first = cells[0]
  let firstDistance = Infinity
  for (const cell of cells) {
    const dx = cell % width - sumColumn / cells.length
    const dz = Math.floor(cell / width) - sumRow / cells.length
    const distance = dx * dx + dz * dz
    if (distance < firstDistance) { first = cell; firstDistance = distance }
  }
  const seeds = [first]
  while (seeds.length < count) {
    let best = cells[0]
    let bestDistance = -1
    for (const cell of cells) {
      let nearest = Infinity
      for (const seed of seeds) {
        const dx = cell % width - seed % width
        const dz = Math.floor(cell / width) - Math.floor(seed / width)
        nearest = Math.min(nearest, dx * dx + dz * dz)
      }
      if (nearest > bestDistance) { best = cell; bestDistance = nearest }
    }
    seeds.push(best)
  }
  return seeds
}

function visible(blocked: Uint8Array, width: number, source: number, x: number, z: number): boolean {
  let column = source % width
  let row = Math.floor(source / width)
  const targetColumn = Math.floor(x)
  const targetRow = Math.floor(z)
  const dx = x - column - 0.5
  const dz = z - row - 0.5
  const stepX = Math.sign(dx)
  const stepZ = Math.sign(dz)
  const deltaX = dx === 0 ? Infinity : Math.abs(1 / dx)
  const deltaZ = dz === 0 ? Infinity : Math.abs(1 / dz)
  let crossX = dx === 0 ? Infinity : 0.5 * deltaX
  let crossZ = dz === 0 ? Infinity : 0.5 * deltaZ
  while (column !== targetColumn || row !== targetRow) {
    if (crossX < crossZ) { column += stepX; crossX += deltaX }
    else if (crossZ < crossX) { row += stepZ; crossZ += deltaZ }
    else {
      if (blocked[row * width + column + stepX]
        || blocked[(row + stepZ) * width + column]) return false
      column += stepX; row += stepZ; crossX += deltaX; crossZ += deltaZ
    }
    if (blocked[row * width + column]) return false
  }
  return true
}

class SnapshotFireLighting implements FireLightingModel {
  readonly anchors: readonly FireLightAnchor[]
  private readonly sources: readonly number[]
  private readonly blocked: Uint8Array
  private readonly width: number
  private readonly height: number

  constructor(
    anchors: readonly FireLightAnchor[], sources: readonly number[], blocked: Uint8Array,
    width: number, height: number,
  ) {
    this.anchors = anchors
    this.sources = sources
    this.blocked = blocked
    this.width = width
    this.height = height
  }

  sample(x: number, z: number): number {
    if (!Number.isFinite(x) || !Number.isFinite(z)
      || x < 0 || z < 0 || x >= this.width || z >= this.height) return 0
    let amount = 0
    for (const source of this.sources) {
      const dx = x - source % this.width - 0.5
      const dz = z - Math.floor(source / this.width) - 0.5
      const distance = Math.hypot(dx, dz)
      if (distance >= LIGHT_RADIUS || !visible(this.blocked, this.width, source, x, z)) continue
      const falloff = 1 - distance / LIGHT_RADIUS
      amount += 0.78 * falloff * falloff * (1 - amount)
    }
    return Math.min(1, amount)
  }
}

/** Builds a static, snapshot-bound model exclusively from public fire and board facts. */
export function createFireLighting(snapshot: SnapshotMessage): FireLightingModel {
  const { width, height } = snapshot.state.arena
  const area = Math.min(width * height, MAX_CELLS)
  const fire = publicFire(snapshot, area)
  const seeds = anchorCounts(components(fire.mask, fire.cells, width))
    .flatMap(({ cells, count }) => representatives(cells, count, width))
    .sort((left, right) => left - right)
  const intensity = Math.min(0.9, 2.4 / Math.max(1, seeds.length))
  const anchors = seeds.map((cell): FireLightAnchor => {
    const column = cell % width
    const row = Math.floor(cell / width)
    return {
      id: `fire:${cell}`,
      position: [column + 0.5, 0.7, row + 0.5],
      color: COLORS[cell % COLORS.length], intensity, distance: LIGHT_RADIUS,
    }
  })
  return new SnapshotFireLighting(anchors, fire.cells, blockers(snapshot, area), width, height)
}
