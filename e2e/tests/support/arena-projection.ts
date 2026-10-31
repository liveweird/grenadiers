export interface PixelRectangle {
  x: number
  y: number
  width: number
  height: number
}

interface ArenaProjection {
  board?: { width: number, height: number }
  viewport: { width: number, height: number }
}

const CAMERA_ELEVATION = 55 * Math.PI / 180
const MAX_SCENERY_HEIGHT = 1.1
const FIT_PADDING = 1.13
const FOOTPRINT_HALF_WIDTH = 0.42
const FOOTPRINT_HALF_DEPTH = 0.32
const FOOTPRINT_BOTTOM = 0.02
const FOOTPRINT_TOP = 1.3

function projectedPoint(
  projection: ArenaProjection, world: { x: number, y: number, z: number },
): { x: number, y: number } {
  const board = projection.board ?? { width: 15, height: 13 }
  const { width, height } = projection.viewport
  const aspect = Math.max(1, width) / Math.max(1, height)
  const projectedHeight = (board.height + 2.2) * Math.sin(CAMERA_ELEVATION)
    + MAX_SCENERY_HEIGHT * Math.cos(CAMERA_ELEVATION)
  const viewHeight = Math.max(projectedHeight, (board.width + 2.2) / aspect) * FIT_PADDING
  const pixelsPerWorldUnit = height / viewHeight
  return {
    x: width / 2 + (world.x - board.width / 2) * pixelsPerWorldUnit,
    y: height / 2 + (world.z - board.height / 2) * Math.sin(CAMERA_ELEVATION)
      * pixelsPerWorldUnit - world.y * Math.cos(CAMERA_ELEVATION) * pixelsPerWorldUnit,
  }
}

/** Screen-space footprint occupied by a billboard standing at a board cell center. */
export function projectedCellFootprint(
  viewport: { width: number, height: number }, column: number, row: number,
  board: { width: number, height: number } = { width: 15, height: 13 },
): PixelRectangle {
  const projection = { board, viewport }
  const points = [FOOTPRINT_BOTTOM, FOOTPRINT_TOP].flatMap((y) =>
    [-FOOTPRINT_HALF_WIDTH, FOOTPRINT_HALF_WIDTH].flatMap((x) =>
      [-FOOTPRINT_HALF_DEPTH, FOOTPRINT_HALF_DEPTH].map((z) => projectedPoint(projection, {
        x: column + 0.5 + x, y, z: row + 0.5 + z,
      }))))
  const left = Math.max(0, Math.floor(Math.min(...points.map(({ x }) => x))))
  const top = Math.max(0, Math.floor(Math.min(...points.map(({ y }) => y))))
  const right = Math.min(viewport.width, Math.ceil(Math.max(...points.map(({ x }) => x))))
  const bottom = Math.min(viewport.height, Math.ceil(Math.max(...points.map(({ y }) => y))))
  return { x: left, y: top, width: Math.max(1, right - left), height: Math.max(1, bottom - top) }
}
