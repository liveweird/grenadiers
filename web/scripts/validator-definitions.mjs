export const validatorDefinitions = {
  'cpu-debug': {
    schema: '../api/cpu-debug.schema.json',
    draft2020: true,
    declaration: `import type { ErrorObject } from 'ajv'
import type { CpuDebugResponse } from '../../game/client/cpuDebugTypes'

declare const validate: {
  (value: unknown): value is CpuDebugResponse
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile: (ajv, schema) => ajv.compile(schema),
  },
  'gameplay-v21': {
    schema: '../api/gameplay-v21.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v21/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v20': {
    schema: '../api/gameplay-v20.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v20/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v19': {
    schema: '../api/gameplay-v19.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v19/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v18': {
    schema: '../api/gameplay-v18.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v18/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v17': {
    schema: '../api/gameplay-v17.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v17/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v16': {
    schema: '../api/gameplay-v16.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v16/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v15': {
    schema: '../api/gameplay-v15.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v15/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v14': {
    schema: '../api/gameplay-v14.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v14/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v13': {
    schema: '../api/gameplay-v13.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v13/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v12': {
    schema: '../api/gameplay-v12.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v12/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  'gameplay-v11': {
    schema: '../api/gameplay-v11.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { GameServerMessage } from '../../game/v11/schema'

declare const validate: {
  (value: unknown): value is GameServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile(ajv, schema) {
      ajv.addSchema(schema)
      const reference = `${schema.$id}#/$defs/GameServerMessage`
      const validate = ajv.getSchema(reference)
      if (!validate) throw new Error(`AJV did not resolve ${reference}`)
      return validate
    },
  },
  realtime: {
    schema: '../api/realtime.schema.json',
    declaration: `import type { ErrorObject } from 'ajv'
import type { ServerMessage } from '../../realtime/schema'

declare const validate: {
  (value: unknown): value is ServerMessage
  errors?: ErrorObject[] | null
}

export default validate
`,
    compile: (ajv, schema) => ajv.compile(schema),
  },
}

