import { expect, type Page } from '@playwright/test'

export type ActorStatsExpectation = {
  available: string
  capacity: string
  range: string
  speed: string
  equipment: string
  weapon: string
  effects: string
  damageGrace: string
}

export const EMPTY_ACTIVE_ACTOR_STATS = {
  equipment: 'None',
  weapon: 'None',
  effects: 'None',
  damageGrace: '0:00',
} as const

const LABELS: Record<keyof ActorStatsExpectation, string> = {
  available: 'Bombs available',
  capacity: 'Bomb capacity',
  range: 'Blast range',
  speed: 'Speed level',
  equipment: 'Equipment',
  weapon: 'Bomb type',
  effects: 'Effects',
  damageGrace: 'Damage grace',
}

export async function expectActorStats(page: Page, player: 'one' | 'two',
  expected: ActorStatsExpectation): Promise<void> {
  const stats = page.getByLabel(`Player ${player} upgrades`)
  await expect(stats.locator('dt')).toHaveText(Object.values(LABELS))
  const fields = Object.keys(LABELS) as Array<keyof ActorStatsExpectation>
  await expect(stats.locator('dd')).toHaveText(fields.map((field) => expected[field]))
}
