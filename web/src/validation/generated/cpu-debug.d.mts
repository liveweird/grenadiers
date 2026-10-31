import type { ErrorObject } from 'ajv'
import type { CpuDebugResponse } from '../../game/client/cpuDebugTypes'

declare const validate: {
  (value: unknown): value is CpuDebugResponse
  errors?: ErrorObject[] | null
}

export default validate
