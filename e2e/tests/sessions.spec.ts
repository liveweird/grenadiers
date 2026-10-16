import { expect, test, type APIResponse } from '@playwright/test'

async function expectProblem(response: APIResponse, status: number, detail: string): Promise<void> {
  expect(response.status()).toBe(status)
  expect(response.headers()['cache-control']).toBe('no-store')
  expect(response.headers()['content-type']).toContain('application/problem+json')
  expect(await response.json()).toEqual({
    type: 'about:blank',
    title: status === 403 ? 'Forbidden' : 'Bad Request',
    status,
    detail,
  })
}

test('browser session issuance enforces origin and cache policy', async ({ page, request }) => {
  await page.goto('/')
  const issued = await page.evaluate(async () => {
    const results = []
    for (let index = 0; index < 2; index += 1) {
      const response = await fetch('/api/v1/game/session', { method: 'POST' })
      const body: unknown = await response.json()
      results.push({
        status: response.status,
        cacheControl: response.headers.get('cache-control'),
        contentType: response.headers.get('content-type'),
        location: response.headers.get('location'),
        body,
      })
    }
    return results
  })
  for (const response of issued) {
    expect(response.status).toBe(201)
    expect(response.cacheControl).toBe('no-store')
    expect(response.contentType).toContain('application/json')
    expect(response.location).toBeNull()
    expect(response.body).toEqual({ token: expect.stringMatching(/^[0-9a-f]{64}$/) })
  }
  expect(issued[0].body).not.toEqual(issued[1].body)
  expect(await page.context().cookies()).toEqual([])

  await expectProblem(await request.post('/api/v1/game/session'), 403, 'Origin is not allowed')
  await expectProblem(await request.post('/api/v1/game/session', {
    headers: { Origin: 'https://untrusted.example' },
  }), 403, 'Origin is not allowed')
  await expectProblem(await request.post('/api/v1/game/session?unexpected=value', {
    headers: { Origin: new URL(page.url()).origin },
  }), 400, 'Query parameters are not supported')
})
