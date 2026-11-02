import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v20/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
