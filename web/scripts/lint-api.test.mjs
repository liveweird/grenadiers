import assert from 'node:assert/strict'
import { describe, test } from 'vitest'
import { lintApi } from './lint-api.mjs'

function contract({ path = '/api/v1/socket', description = 'Open the game socket.', status = '101' } = {}) {
  return JSON.stringify({
    openapi: '3.0.3',
    info: { title: 'Test API', version: '1.0.0' },
    paths: {
      [path]: {
        get: {
          ...(description === null ? {} : { description }),
          operationId: 'openSocket',
          tags: ['game'],
          responses: { [status]: { description: 'Successful response' } },
        },
      },
    },
  })
}

async function errorCodes(source) {
  return (await lintApi(source, 'test-openapi.json'))
    .filter((diagnostic) => diagnostic.severity === 0)
    .map((diagnostic) => diagnostic.code)
}

describe('API lint guardrails', () => {
  test('accepts a versioned WebSocket operation whose success response is 101', async () => {
    assert.deepEqual(await errorCodes(contract()), [])
  })

  test('rejects an unversioned operation through the custom versioned-paths rule', async () => {
    assert((await errorCodes(contract({ path: '/socket' }))).includes('versioned-paths'))
  })

  test('requires operation descriptions', async () => {
    assert((await errorCodes(contract({ description: null }))).includes('operation-description'))
  })

  test('fails malformed contracts through parser diagnostics', async () => {
    assert((await errorCodes('{')).includes('parser'))
  })
})
