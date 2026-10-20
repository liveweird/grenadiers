import { describe, expect, it, vi } from 'vitest'
import { catalogItemsForMask, checkedCatalogMaskRanges, executeCatalogMaskRange } from './catalogMasks'

const expected = ['a', 'b', 'c'] as const
const plan = (catalog: readonly string[] = expected, rangeSize = 10) => ({
  protocol: 'test protocol', catalog, expectedCount: 3, expectedOrder: expected, rangeSize,
})

describe('catalog mask batching', () => {
  it('builds contiguous disjoint half-open ranges with a short final range', () => {
    expect(checkedCatalogMaskRanges(plan())).toEqual([{ start: 0, end: 8 }])
    const five = ['a', 'b', 'c', 'd', 'e'] as const
    const ranges = checkedCatalogMaskRanges({
      protocol: 'five', catalog: five, expectedCount: 5, expectedOrder: five, rangeSize: 10,
    })
    expect(ranges).toEqual([
      { start: 0, end: 10 }, { start: 10, end: 20 },
      { start: 20, end: 30 }, { start: 30, end: 32 },
    ])
  })

  it('visits every mask exactly once in progression without retaining a visited matrix', () => {
    let next = 0
    let count = 0
    for (const range of checkedCatalogMaskRanges(plan(expected, 3))) {
      executeCatalogMaskRange('test protocol', expected, range, (mask) => {
        expect(mask).toBe(next)
        next += 1
        count += 1
      })
    }
    expect({ next, count }).toEqual({ next: 8, count: 8 })
  })

  it('matches the dense filter mapping exhaustively for sixteen catalog entries', () => {
    const catalog = Array.from({ length: 16 }, (_, index) => `item-${index}`)
    for (let mask = 0; mask < 2 ** catalog.length; mask += 1) {
      expect(catalogItemsForMask(catalog, mask)).toEqual(
        catalog.filter((_, index) => (mask & (1 << index)) !== 0),
      )
    }
  })

  it('preserves canonical order for empty, singleton and full masks', () => {
    expect(catalogItemsForMask(expected, 0)).toEqual([])
    expect(catalogItemsForMask(expected, 2)).toEqual(['b'])
    expect(catalogItemsForMask(expected, 7)).toEqual(expected)
  })

  it.each([
    ['duplicate', ['a', 'b', 'b'], 'items must be unique'],
    ['missing', ['a', 'b'], 'must contain 3 items; got 2'],
    ['reordered', ['b', 'a', 'c'], 'item 0 must be a; got b'],
    ['enlarged', ['a', 'b', 'c', 'd'], 'must contain 3 items; got 4'],
  ])('rejects a %s catalog before calling the range enumerator', (_label, catalog, message) => {
    const enumerate = vi.fn()
    expect(() => checkedCatalogMaskRanges(plan(catalog), enumerate)).toThrow(message)
    expect(enumerate).not.toHaveBeenCalled()
  })

  it('continues after failures and reports bounded first-failure diagnostics', () => {
    const attempted: number[] = []
    let thrown: Error | null = null
    try {
      executeCatalogMaskRange('test protocol', expected, { start: 0, end: 8 }, (mask) => {
        attempted.push(mask)
        if ([1, 4, 7].includes(mask)) throw new Error(`failure ${mask} ${'x'.repeat(1_000)}`)
      })
    } catch (error) {
      if (error instanceof Error) thrown = error
    }
    expect(attempted).toEqual([0, 1, 2, 3, 4, 5, 6, 7])
    expect(thrown?.message).toContain(
      'test protocol catalog masks [0, 8) attempted=8 failed=3; first mask=1 (0x1) enabled=["a"]',
    )
    expect(thrown?.message).toContain('omitted=2')
    expect(thrown?.message.length).toBeLessThan(500)
  })

  it('allows a later range to execute after an earlier range fails', () => {
    const later = vi.fn()
    expect(() => executeCatalogMaskRange(
      'test protocol', expected, { start: 0, end: 2 }, () => { throw new Error('first') },
    )).toThrow('failed=2')
    executeCatalogMaskRange('test protocol', expected, { start: 2, end: 4 }, later)
    expect(later).toHaveBeenCalledTimes(2)
  })
})
