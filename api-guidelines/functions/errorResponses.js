const HTTP_METHODS = new Set([
  'delete',
  'get',
  'head',
  'options',
  'patch',
  'post',
  'put',
  'trace',
])

const PROBLEM_SCHEMA_REF = '#/components/schemas/Problem'

function isPlainTextUpgradeException(path, method, status, response) {
  if (path !== '/api/v1/ws' || method !== 'get' || status !== '400') return false

  const content = response?.content
  if (!content || Object.keys(content).length !== 1) return false

  const schema = content['text/plain']?.schema
  return schema?.type === 'string' && Object.keys(schema).length === 1
}

function validateProblemResponse(response, path) {
  const mediaType = response?.content?.['application/problem+json']
  if (!mediaType) {
    return [{
      message: 'Documented 4xx and 5xx responses must provide application/problem+json.',
      path: [...path, 'content'],
    }]
  }

  if (mediaType.schema?.$ref !== PROBLEM_SCHEMA_REF) {
    return [{
      message: `application/problem+json must reference ${PROBLEM_SCHEMA_REF}.`,
      path: [...path, 'content', 'application/problem+json', 'schema'],
    }]
  }

  return []
}

function validateResponseOrReference(document, response, path, validatedComponents) {
  if (typeof response?.$ref !== 'string') return validateProblemResponse(response, path)

  const match = /^#\/components\/responses\/([^/]+)$/.exec(response.$ref)
  if (!match) {
    return [{
      message: 'Documented error response references must target a local component response.',
      path: [...path, '$ref'],
    }]
  }

  const name = decodeURIComponent(match[1])
  if (validatedComponents.has(name)) return []
  validatedComponents.add(name)

  const component = document?.components?.responses?.[name]
  if (!component) return [] // The base OpenAPI rules report unresolved references.
  return validateProblemResponse(component, ['components', 'responses', name])
}

export default function errorResponses(_resolvedDocument, _options, context) {
  const document = context.document.data
  const errors = []
  const validatedComponents = new Set()

  for (const [apiPath, pathItem] of Object.entries(document?.paths ?? {})) {
    for (const [method, operation] of Object.entries(pathItem ?? {})) {
      if (!HTTP_METHODS.has(method)) continue

      for (const [status, response] of Object.entries(operation?.responses ?? {})) {
        if (!/^[45][0-9]{2}$/.test(status)) continue
        if (isPlainTextUpgradeException(apiPath, method, status, response)) continue

        errors.push(...validateResponseOrReference(
          document,
          response,
          ['paths', apiPath, method, 'responses', status],
          validatedComponents,
        ))
      }
    }
  }

  for (const [status, response] of Object.entries(document?.['x-api-fallback-responses'] ?? {})) {
    if (!/^[45][0-9]{2}$/.test(status)) continue
    errors.push(...validateResponseOrReference(
      document,
      response,
      ['x-api-fallback-responses', status],
      validatedComponents,
    ))
  }

  return errors
}
