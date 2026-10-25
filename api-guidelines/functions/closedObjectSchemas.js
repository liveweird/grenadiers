import { visitSchemas } from './schemaVisitor.js'

export default function closedObjectSchemas(_resolvedDocument, _options, context) {
  const document = context.document.data
  const errors = []
  visitSchemas(document, (schema, path) => {
    // A direct allOf fragment may intentionally describe only part of its parent.
    // Only a typed, closed parent can safely carry that fragment's object boundary.
    const allOfParentPath = path.slice(0, -2)
    const allOfParent = allOfParentPath.reduce((value, key) => value?.[key], document)
    const hasClosedAllOfParent = path.at(-2) === 'allOf'
      && allOfParent?.type === 'object'
      && allOfParent.additionalProperties === false
    const hasObjectKeywords = Object.hasOwn(schema, 'properties')
      || Object.hasOwn(schema, 'required')
    const isObjectShaped = schema.type === 'object'
      || (hasObjectKeywords && !hasClosedAllOfParent)
    if (!isObjectShaped) return

    if (schema.type !== 'object') {
      errors.push({
        message: 'Schemas with properties or required fields must set type to object.',
        path: [...path, 'type'],
      })
    }
    if (schema.additionalProperties !== false) {
      errors.push({
        message: 'Object schemas must set additionalProperties to false.',
        path: [...path, 'additionalProperties'],
      })
    }
  })
  return errors
}
