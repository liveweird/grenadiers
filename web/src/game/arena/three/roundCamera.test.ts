import { describe, expect, it, vi } from 'vitest'
import { Vector3 } from 'three'
import { gameSnapshot } from '../../../test/game-snapshot'
import { createArenaCamera, fitArenaCamera } from './camera'
import {
  createRoundCameraController, ROUND_CAMERA_DURATION_MS,
} from './roundCamera'

const board = { width: 15, height: 13 } as const
const running = (roundId: number, tick: number) => {
  const snapshot = gameSnapshot()
  snapshot.room.roundId = roundId
  snapshot.room.phase = { type: 'running' }
  snapshot.state.physicalTick = tick
  return snapshot
}
const opening = (roundId: number, tick: number, bornAt: number, key = `opening:${roundId}`) => {
  const snapshot = running(roundId, tick)
  snapshot.presentationOpening = { key, bornAt }
  return snapshot
}

describe('round camera effects', () => {
  it.each([
    [0, 'zoomIn'], [0.26, 'zoomOut'], [0.51, 'rotateLeft'], [0.99, 'rotateRight'],
  ] as const)('selects each variant uniformly and eases an opening to exact neutral', (roll, variant) => {
    const random = vi.fn(() => roll)
    const controller = createRoundCameraController(random)
    const camera = createArenaCamera()
    fitArenaCamera(camera, board, { width: 900, height: 600 })
    const canonicalPosition = camera.position.clone()
    const canonicalQuaternion = camera.quaternion.clone()

    expect(controller.update(opening(1, 0, 100), 100)).toBe(variant)
    controller.apply(camera, board, 100)
    const startingZoom = camera.zoom
    if (variant === 'zoomIn') expect(startingZoom).toBeLessThan(1)
    if (variant === 'zoomOut') expect(startingZoom).toBeGreaterThan(1)
    if (variant === 'rotateLeft') expect(camera.position.x).toBeLessThan(canonicalPosition.x)
    if (variant === 'rotateRight') expect(camera.position.x).toBeGreaterThan(canonicalPosition.x)
    controller.apply(camera, board, 100 + ROUND_CAMERA_DURATION_MS / 2)
    if (variant === 'zoomIn') expect(camera.zoom).toBeGreaterThan(startingZoom)
    if (variant === 'zoomOut') expect(camera.zoom).toBeLessThan(startingZoom)
    controller.apply(camera, board, 100 + ROUND_CAMERA_DURATION_MS)
    expect(camera.zoom).toBe(1)
    expect(camera.position.equals(canonicalPosition)).toBe(true)
    expect(camera.quaternion.equals(canonicalQuaternion)).toBe(true)
    expect(random).toHaveBeenCalledOnce()
  })

  it('pulses only once across an observed running result boundary', () => {
    const random = vi.fn(() => 0)
    const controller = createRoundCameraController(random)
    const camera = createArenaCamera()
    fitArenaCamera(camera, board, { width: 900, height: 600 })
    expect(controller.update(running(1, 10), 0)).toBeNull()
    const result = running(1, 11)
    result.room.phase = { type: 'result' }
    expect(controller.update(result, 50)).toBe('zoomIn')
    controller.apply(camera, board, 50)
    expect(camera.zoom).toBe(1)
    controller.apply(camera, board, 50 + ROUND_CAMERA_DURATION_MS / 2)
    expect(camera.zoom).toBeGreaterThan(1)
    expect(controller.update(result, 100)).toBeNull()
    const matchResult = structuredClone(result)
    matchResult.room.phase = { type: 'matchResult' }
    expect(controller.update(matchResult, 150)).toBeNull()
    expect(random).toHaveBeenCalledOnce()
  })

  it('keeps a certified opening through running updates and silently seeds admissions', () => {
    const random = vi.fn(() => 0.99)
    const controller = createRoundCameraController(random)
    const camera = createArenaCamera()
    fitArenaCamera(camera, board, { width: 900, height: 600 })
    expect(controller.update(running(1, 10), 0)).toBeNull()
    expect(controller.update(opening(3, 0, 20), 20)).toBe('rotateRight')
    controller.apply(camera, board, 20)
    expect(camera.position.x).toBeGreaterThan(board.width / 2)
    controller.update(running(3, 1), 40)
    controller.apply(camera, board, 40)
    expect(camera.position.x).toBeGreaterThan(board.width / 2)

    const reconnected = opening(4, 0, 60)
    reconnected.room.presentationBaseline = 'reconnected'
    expect(controller.update(reconnected, 60)).toBeNull()
    controller.apply(camera, board, 60)
    expect(camera.position.x).toBe(board.width / 2)
    expect(random).toHaveBeenCalledOnce()
  })

  it('suppresses stale, late, cleared, and reduced-motion boundaries without picking', () => {
    const random = vi.fn(() => 0)
    const controller = createRoundCameraController(random)
    const camera = createArenaCamera()
    fitArenaCamera(camera, board, { width: 900, height: 600 })
    controller.update(running(1, 10), 0)
    const late = running(1, 11)
    late.room.phase = { type: 'result' }
    expect(controller.update(late, 1_001)).toBeNull()
    controller.update(running(1, 20), 1_010)
    const backward = running(1, 19)
    backward.room.phase = { type: 'result' }
    expect(controller.update(backward, 1_020)).toBeNull()

    controller.setReducedMotion(true, camera, board)
    controller.update(opening(2, 1, 1_040, 'opening:reduced'), 1_040)
    const result = running(2, 2)
    result.room.phase = { type: 'result' }
    expect(controller.update(result, 1_050)).toBeNull()
    controller.setReducedMotion(false, camera, board)
    expect(controller.update(opening(2, 3, 1_040, 'opening:reduced'), 1_060)).toBeNull()
    const staleController = createRoundCameraController(random)
    expect(staleController.update(opening(3, 0, 0), ROUND_CAMERA_DURATION_MS)).toBeNull()
    expect(camera.zoom).toBe(1)
    expect(random).not.toHaveBeenCalled()
  })

  it('allows a novel post-suspend cue but never replays a consumed retained cue', () => {
    const random = vi.fn(() => 0)
    const controller = createRoundCameraController(random)
    const camera = createArenaCamera()
    fitArenaCamera(camera, board, { width: 900, height: 600 })
    controller.clear(camera, board)
    const certified = opening(1, 0, 100)
    expect(controller.update(certified, 100)).toBe('zoomIn')
    controller.clear(camera, board)
    expect(controller.update(certified, 120)).toBeNull()
    expect(random).toHaveBeenCalledOnce()
  })

  it('keeps the board center and elevation while reapplying the sampled resize offset', () => {
    const controller = createRoundCameraController(() => 0.99)
    const camera = createArenaCamera()
    fitArenaCamera(camera, board, { width: 900, height: 600 })
    controller.update(opening(1, 0, 0), 0)
    controller.apply(camera, board, 0)
    const position = camera.position.clone()
    const direction = camera.getWorldDirection(new Vector3())
    const quaternion = camera.quaternion.clone()
    fitArenaCamera(camera, board, { width: 320, height: 700 })
    controller.reapply(camera, board)
    expect(camera.position.equals(position)).toBe(true)
    expect(camera.getWorldDirection(new Vector3()).equals(direction)).toBe(true)
    expect(camera.quaternion.equals(quaternion)).toBe(true)
  })
})
