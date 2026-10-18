import { expect, type Page } from '@playwright/test'
import type { ObservedGame } from './duel'

/** Preserve the original seeded scenarios while exercising the real settings UI. */
export async function selectClassicSettings(page: Page, game: ObservedGame): Promise<void> {
  const room = () => game.frames.filter((frame) =>
    frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room
  await expect.poll(() => room()?.settings.enabledItems).toEqual(['heart'])
  const before = room()!.settingsRevision
  await page.getByRole('button', { name: 'Classic settings', exact: true }).click()
  await page.getByRole('button', { name: 'Apply rules', exact: true }).click()
  await expect.poll(() => room()?.settingsRevision).toBeGreaterThan(before)
  expect(room()?.settings.enabledItems).toEqual([])
  expect(room()).toMatchObject({ rulesVersion: 5, mapId: 'generated_v2' })
}
