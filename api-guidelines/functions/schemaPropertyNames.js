import { visitSchemas } from './schemaVisitor.js'

const CAMEL_CASE = /^[a-z][A-Za-z0-9]*$/

export default function schemaPropertyNames(_resolvedDocument, _options, context) {
  const errors = []
  visitSchemas(context.document.data, (schema, path) => {
    if (schema.properties !== null && typeof schema.properties === 'object') {
      for (const propertyName of Object.keys(schema.properties)) {
        if (!CAMEL_CASE.test(propertyName)) {
          errors.push({
            message: `JSON property "${propertyName}" must use camelCase.`,
            path: [...path, 'properties', propertyName],
          })
        }
      }
    }
  })
  return errors
}
