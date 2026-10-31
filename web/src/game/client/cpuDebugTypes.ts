import type { paths } from '../../api/schema'

type CpuDebugOperation = paths['/api/v1/game/cpu-debug']['post']

export type CpuDebugRequest = NonNullable<CpuDebugOperation['requestBody']>['content']['application/json']
export type CpuDebugResponse = CpuDebugOperation['responses'][200]['content']['application/json']
export type CpuDebugRow = CpuDebugResponse['cpus'][number]
export type AvailableCpuDebugRow = Extract<CpuDebugRow, { status: 'available' }>

export interface CpuDebugBaseline {
  roomId: string
  roundId: number
  tick: number
  revision: number
  youSeat: CpuDebugResponse['youSeat']
}

export type CpuDebugHttpResult =
  | { type: 'response'; response: Response; baseline: CpuDebugBaseline }
  | { type: 'stale' }
  | { type: 'unavailable' }
