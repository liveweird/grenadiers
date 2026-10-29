import { expect, type Page } from '@playwright/test'
import type { ObservedGame } from './duel'

type ItemSelection = { id: string; label: string }

const latestRoom = (game: ObservedGame) => game.frames.filter((frame) =>
  frame.type === 'admitted' || frame.type === 'roomState' || frame.type === 'snapshot').at(-1)?.room

export async function applyItemSettings(
  page: Page,
  game: ObservedGame,
  items: ItemSelection[],
  editDraft?: () => Promise<unknown>,
): Promise<void> {
  await expect.poll(() => latestRoom(game)).toBeTruthy()
  const before = latestRoom(game)!.settingsRevision
  await page.getByRole('button', { name: 'All off', exact: true }).click()
  for (const item of items) {
    await page.getByRole('checkbox', { name: item.label, exact: true }).check()
  }
  await editDraft?.()
  await page.getByRole('button', { name: 'Apply rules', exact: true }).click()
  await expect.poll(() => latestRoom(game)?.settingsRevision).toBeGreaterThan(before)
  expect(latestRoom(game)?.settings.enabledItems).toEqual(items.map(({ id }) => id))
  expect(latestRoom(game)?.settings.environment).toEqual({ stormCloud: false })
  expect(latestRoom(game)?.configuration.environment).toEqual({ stormCloud: false })
}

/** Preserve the original seeded scenarios while exercising the real settings UI. */
export async function selectClassicSettings(page: Page, game: ObservedGame): Promise<void> {
  await expect.poll(() => latestRoom(game)).toBeTruthy()
  const before = latestRoom(game)!.settingsRevision
  await page.getByRole('button', { name: 'Classic settings', exact: true }).click()
  await page.getByRole('button', { name: 'Apply rules', exact: true }).click()
  await expect.poll(() => latestRoom(game)?.settingsRevision).toBeGreaterThan(before)
  expect(latestRoom(game)?.settings.enabledItems).toEqual([])
  expect(latestRoom(game)).toMatchObject({
    rulesVersion: 7, mapId: 'generated_v3', settings: { environment: { stormCloud: false } },
  })
}
