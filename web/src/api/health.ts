import type { paths } from './schema'

export type HealthResponse =
  paths['/api/v1/health']['get']['responses'][200]['content']['application/json']

export async function getHealth(signal?: AbortSignal): Promise<HealthResponse> {
  const response = await fetch('/api/v1/health', signal ? { signal } : {})
  if (!response.ok) throw new Error(`Health request failed (${response.status})`)
  return response.json() as Promise<HealthResponse>
}
