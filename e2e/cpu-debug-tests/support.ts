import { expect, type Page } from '@playwright/test'
import { latestSnapshot, type ObservedGame } from '../tests/support/duel'
import type { CpuDebugResponse } from '../../web/src/game/client/cpuDebugTypes'
import validateCpuDebug from '../../web/src/validation/generated/cpu-debug.mjs'

export const debugPath = '/api/v1/game/cpu-debug'
export const englishTitle = 'CPU intent diagnostics'
export const polishTitle = 'Diagnostyka zamiarów komputera'

export function observeDebug(page: Page) {
  const responses: CpuDebugResponse[] = []
  const statuses: number[] = []
  let requests = 0
  page.on('request', (request) => {
    if (new URL(request.url()).pathname === debugPath) requests++
  })
  page.on('response', async (response) => {
    if (new URL(response.url()).pathname !== debugPath) return
    statuses.push(response.status())
    if (response.status() !== 200) return
    const value: unknown = await response.json().catch(() => null)
    if (validateCpuDebug(value)) responses.push(value)
  })
  return { responses, statuses, requestCount: () => requests }
}

export async function createCpuMatch(page: Page, game: ObservedGame, start: boolean) {
  await page.getByRole('combobox', { name: 'Players', exact: true }).selectOption('4')
  for (const index of [2, 3, 4]) {
    await page.getByRole('combobox', { name: `Player ${index} controller`, exact: true }).selectOption('cpu')
  }
  await page.getByRole('combobox', { name: 'CPU difficulty', exact: true }).last().selectOption('standard')
  await page.getByRole('button', { name: 'Create custom match', exact: true }).click()
  await page.getByRole('button', { name: 'All off', exact: true }).click()
  await page.getByRole('button', { name: 'Apply rules', exact: true }).click()
  if (!start) return
  await page.getByRole('button', { name: 'Ready', exact: true }).click()
  await expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')
}
