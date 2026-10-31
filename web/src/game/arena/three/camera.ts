import { OrthographicCamera } from 'three'
import { WORLD_TILE } from './coordinates'

const ELEVATION = 55 * Math.PI / 180
const MAX_SCENERY_HEIGHT = 1.1
const FIT_PADDING = 1.13

export function createArenaCamera(): OrthographicCamera {
  const camera = new OrthographicCamera(-1, 1, 1, -1, 1, 100_000)
  camera.name = 'arena-camera'
  return camera
}

export function fitArenaCamera(
  camera: OrthographicCamera,
  board: { width: number; height: number },
  viewport: { width: number; height: number },
): void {
  const boardWidth = (board.width + 2.2) * WORLD_TILE
  const boardDepth = (board.height + 2.2) * WORLD_TILE
  const projectedHeight = boardDepth * Math.sin(ELEVATION) + MAX_SCENERY_HEIGHT * Math.cos(ELEVATION)
  const aspect = Math.max(1, viewport.width) / Math.max(1, viewport.height)
  const viewHeight = Math.max(projectedHeight, boardWidth / aspect) * FIT_PADDING
  const viewWidth = viewHeight * aspect
  camera.left = -viewWidth / 2
  camera.right = viewWidth / 2
  camera.top = viewHeight / 2
  camera.bottom = -viewHeight / 2
  const distance = 26
  const centerX = board.width * WORLD_TILE / 2
  const centerZ = board.height * WORLD_TILE / 2
  camera.position.set(
    centerX, Math.sin(ELEVATION) * distance, centerZ + Math.cos(ELEVATION) * distance,
  )
  camera.lookAt(centerX, 0, centerZ)
  camera.near = 0.1
  camera.far = 70
  camera.updateProjectionMatrix()
  camera.updateMatrixWorld()
}
