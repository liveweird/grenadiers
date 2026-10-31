import type { ArenaRenderer } from '../src/game/arena/rendererTypes'
import type { SnapshotMessage } from '../src/game/arena/types'
import { threeArenaSnapshot, type BoardName } from './three-arena-snapshots'

export type RoundCameraVariant = 'zoomIn' | 'zoomOut' | 'rotateLeft' | 'rotateRight'
export type RoundCameraBoundary = 'start' | 'end'
const variants: RoundCameraVariant[] = ['zoomIn', 'zoomOut', 'rotateLeft', 'rotateRight']
let selected = 0
export const roundCameraRandom = (): number => (selected + .5) / variants.length

export interface RoundCameraDiagnosticApi {
  prepareRoundCamera(board: BoardName, boundary: RoundCameraBoundary): Promise<void>
  beginRoundCamera(variant: RoundCameraVariant): Promise<number>
  sampleRoundCamera(ageMs: number): Promise<void>
  enterRoundCameraRunning(): Promise<void>
}

export function createRoundCameraDiagnostic(hooks: {
  initialize: () => Promise<unknown>
  renderer: () => ArenaRenderer | null
  setNow: (now: number) => void
  now: () => number
  setBoard: (board: BoardName) => void
  settle: (frames?: number) => Promise<void>
}): RoundCameraDiagnosticApi {
  let snapshot: SnapshotMessage | null = null
  let generation = 0
  let bornAt = 0
  let boundary: RoundCameraBoundary = 'start'
  const update = async (): Promise<void> => {
    if (!snapshot) throw new Error('Prepare camera fixture before updating it')
    hooks.renderer()?.update(structuredClone(snapshot))
    await hooks.settle(2)
  }
  return {
    async prepareRoundCamera(board, nextBoundary) {
      await hooks.initialize()
      hooks.renderer()?.suspend()
      hooks.setBoard(board)
      boundary = nextBoundary
      generation += 1
      snapshot = threeArenaSnapshot(board, 'empty')
      snapshot.room.presentationBaseline = `round-camera-${generation}`
      snapshot.room.phase = { type: boundary === 'start' ? 'waiting' : 'running' }
      snapshot.state.physicalTick = 10
      bornAt = hooks.now()
      hooks.setNow(bornAt)
      await update()
    },
    async beginRoundCamera(variant) {
      if (!snapshot) throw new Error('Prepare camera fixture before starting it')
      selected = variants.indexOf(variant)
      if (selected < 0) throw new Error('Unknown camera variant')
      bornAt += 1
      hooks.setNow(bornAt)
      snapshot.room.phase = { type: boundary === 'start' ? 'running' : 'result' }
      snapshot.state.physicalTick = 11
      if (boundary === 'start') snapshot.presentationOpening = { key: `round-camera-${generation}`, bornAt }
      await update()
      return bornAt
    },
    async sampleRoundCamera(ageMs) {
      hooks.setNow(bornAt + ageMs)
      await hooks.settle(2)
    },
    async enterRoundCameraRunning() {
      if (!snapshot) throw new Error('Prepare camera fixture before running it')
      snapshot.room.phase = { type: 'running' }
      snapshot.state.physicalTick = 12
      await update()
    },
  }
}
