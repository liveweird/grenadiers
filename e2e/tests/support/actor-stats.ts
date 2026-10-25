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
  equipment: 'Gear',
  weapon: 'Bomb type',
  effects: 'Effects',
  damageGrace: 'Damage grace',
}

export async function expectActorStats(page: Page, player: 'one' | 'two', ownSeat: 'one' | 'two',
  expected: ActorStatsExpectation): Promise<void> {
  const fields = Object.keys(LABELS) as Array<keyof ActorStatsExpectation>
  if (player === ownSeat) {
    const stats = page.getByRole('region', { name: 'Your items' })
    await expect(stats.locator('.own-status-item small')).toHaveText(Object.values(LABELS))
    await expect(stats.locator('.own-status-item strong'))
      .toHaveText(fields.map((field) => expected[field]))
    return
  }
  const stats = page.getByLabel(`Player ${player} upgrades`)
  await expect(stats.locator('dt')).toHaveText(Object.values(LABELS))
  await expect(stats.locator('dd')).toHaveText(fields.map((field) => expected[field]))
}

type LabelGeometry = {
  cellCount: number
  columnCount?: number
  fragmentCounts: number[]
  overflow: number[]
  overlaps: string[]
  outsideViewport: number[]
  containerOutsideViewport: boolean
}

export async function expectActorStatGeometry(page: Page, rivalColumnCount: number): Promise<void> {
  const geometry = await page.locator('body').evaluate((body) => {
    const measure = (container: Element, labelSelector: string): LabelGeometry => {
      const cells = [...container.querySelectorAll<HTMLElement>(':scope > div')]
      const labels = cells.map((cell, index) => {
        const range = document.createRange()
        range.selectNodeContents(cell.querySelector(labelSelector)!)
        return { cell: cell.getBoundingClientRect(), index, text: [...range.getClientRects()] }
      })
      const overflow = labels.flatMap(({ cell, index, text }) => text
        .filter(({ left, right }) => left < cell.left - 0.5 || right > cell.right + 0.5)
        .map(() => index))
      const overlaps: string[] = []
      for (const [index, label] of labels.entries()) {
        for (const other of labels.slice(index + 1)) {
          if (label.text.some((first) => other.text.some((second) =>
            first.left < second.right - 0.5 && first.right > second.left + 0.5
            && first.top < second.bottom - 0.5 && first.bottom > second.top + 0.5))) {
            overlaps.push(`${label.index}:${other.index}`)
          }
        }
      }
      const outsideViewport = labels.filter(({ cell }) => cell.width <= 0 || cell.height <= 0
        || cell.left < 0 || cell.right > innerWidth).map(({ index }) => index)
      const bounds = container.getBoundingClientRect()
      return {
        cellCount: cells.length,
        columnCount: new Set(labels.map(({ cell }) => Math.round(cell.left * 2) / 2)).size,
        fragmentCounts: labels.map(({ text }) => text.length), overflow, overlaps, outsideViewport,
        containerOutsideViewport: bounds.width <= 0 || bounds.height <= 0
          || bounds.left < 0 || bounds.right > innerWidth,
      }
    }
    return {
      rival: [...body.querySelectorAll('.actor-stats')].map((grid) => measure(grid, 'dt')),
      own: [...body.querySelectorAll('.own-status-list')].map((list) => measure(list, 'small')),
    }
  })
  const { rival, own } = geometry
  expect(rival).toHaveLength(1)
  expect(rival[0]).toMatchObject({
    cellCount: 8, columnCount: rivalColumnCount, overflow: [], overlaps: [], outsideViewport: [],
    containerOutsideViewport: false,
  })
  expect(rival[0]!.fragmentCounts.every((count) => count > 0)).toBe(true)

  expect(own).toHaveLength(1)
  expect(own[0]).toMatchObject({
    cellCount: 8, overflow: [], overlaps: [], outsideViewport: [], containerOutsideViewport: false,
  })
  expect(own[0]!.fragmentCounts.every((count) => count > 0)).toBe(true)
}
