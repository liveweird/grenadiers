import type { ErrorObject } from 'ajv'
import type { ServerMessage } from '../../realtime/schema'

declare const validate: {
  (value: unknown): value is ServerMessage
  errors?: ErrorObject[] | null
}

export default validate
