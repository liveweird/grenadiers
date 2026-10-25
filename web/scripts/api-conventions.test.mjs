import assert from 'node:assert/strict'
import { readFile } from 'node:fs/promises'
import { resolve } from 'node:path'
import { beforeAll, describe, test } from 'vitest'
import { lintApi } from './lint-api.mjs'

let canonicalContract

beforeAll(async () => {
  const source = await readFile(resolve(import.meta.dirname, '../../api/openapi.json'), 'utf8')
  canonicalContract = JSON.parse(source)
})

async function errorsFor(contract) {
  return (await lintApi(JSON.stringify(contract), 'api-conventions-fixture.json'))
    .filter((diagnostic) => diagnostic.severity === 0)
}

function cloneContract() {
  return structuredClone(canonicalContract)
}

describe('API-STRUCT-001 lint enforcement', () => {
  test('accepts the canonical contract, including the exact malformed-upgrade exception', async () => {
    assert.deepEqual(await errorsFor(cloneContract()), [])
  })

  test('requires problem+json for a documented error response', async () => {
    const contract = cloneContract()
    contract.paths['/api/v1/ready'].get.responses['503'].content = {
      'application/json': {
        schema: { $ref: '#/components/schemas/Problem' },
      },
    }

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'error-problem-details'
      && message.includes('must provide application/problem+json')
    )))
  })

  test('requires error bodies to reference the shared Problem schema', async () => {
    const contract = cloneContract()
    contract.paths['/api/v1/ready'].get.responses['503']
      .content['application/problem+json'].schema = {
        type: 'object',
        additionalProperties: false,
        properties: {},
      }

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'error-problem-details'
      && message.includes('must reference #/components/schemas/Problem')
    )))
  })

  test('checks shared error response components reached through fallback references', async () => {
    const contract = cloneContract()
    contract.components.responses.NotFound.content = {
      'application/json': {
        schema: { $ref: '#/components/schemas/Problem' },
      },
    }

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message, path }) => (
      code === 'error-problem-details'
      && message.includes('must provide application/problem+json')
      && path.join('.') === 'components.responses.NotFound.content'
    )))
  })

  test('requires object schemas to reject undocumented properties', async () => {
    const contract = cloneContract()
    delete contract.components.schemas.Status.additionalProperties

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'closed-object-schemas'
      && message.includes('additionalProperties to false')
    )))
  })

  test('does not allow object keywords to bypass closure by omitting the type', async () => {
    const contract = cloneContract()
    delete contract.components.schemas.Status.type
    delete contract.components.schemas.Status.additionalProperties

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'closed-object-schemas'
      && message.includes('must set type to object')
    )))
    assert(diagnostics.some(({ code, message }) => (
      code === 'closed-object-schemas'
      && message.includes('additionalProperties to false')
    )))
  })

  test('rejects a non-object type on a schema that declares properties', async () => {
    const contract = cloneContract()
    contract.components.schemas.Status.type = 'string'

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'closed-object-schemas'
      && message.includes('must set type to object')
    )))
  })

  test('allows a direct allOf fragment to restrict its closed parent', async () => {
    const contract = cloneContract()
    contract.components.schemas.Status.allOf = [{ required: ['status'] }]

    const diagnostics = await errorsFor(contract)
    assert(!diagnostics.some(({ code }) => code === 'closed-object-schemas'))
  })

  test('does not allow an object-shaped allOf fragment under an unbounded parent', async () => {
    const contract = cloneContract()
    contract.components.schemas.Status = {
      allOf: [{
        required: ['goodName'],
        properties: {
          goodName: { type: 'string' },
        },
      }],
    }

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'closed-object-schemas'
      && message.includes('must set type to object')
    )))
    assert(diagnostics.some(({ code, message }) => (
      code === 'closed-object-schemas'
      && message.includes('additionalProperties to false')
    )))
  })

  test('requires JSON schema properties to use camelCase', async () => {
    const contract = cloneContract()
    const status = contract.components.schemas.Status
    status.required = ['service_status']
    status.properties.service_status = status.properties.status
    delete status.properties.status

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'camel-case-properties'
      && message.includes('"service_status" must use camelCase')
    )))
  })

  test('does not broaden the malformed-upgrade exception to a different schema', async () => {
    const contract = cloneContract()
    contract.paths['/api/v1/ws'].get.responses['400']
      .content['text/plain'].schema.maxLength = 100

    const diagnostics = await errorsFor(contract)
    assert(diagnostics.some(({ code, message }) => (
      code === 'error-problem-details'
      && message.includes('must provide application/problem+json')
    )))
  })
})
