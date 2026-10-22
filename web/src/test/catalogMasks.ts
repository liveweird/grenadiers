export interface CatalogMaskRange {
  start: number
  end: number
}

export interface CatalogMaskPlan<Item extends string> {
  protocol: string
  catalog: readonly Item[]
  expectedCount: number
  expectedOrder: readonly Item[]
  rangeSize?: number
}

type RangeEnumerator = (total: number, rangeSize: number) => CatalogMaskRange[]

const DEFAULT_RANGE_SIZE = 1_024
const MAX_ASSERTION_DETAIL = 240

function enumerateRanges(total: number, rangeSize: number): CatalogMaskRange[] {
  const ranges: CatalogMaskRange[] = []
  for (let start = 0; start < total; start += rangeSize) {
    ranges.push({ start, end: Math.min(start + rangeSize, total) })
  }
  return ranges
}

function assertCatalog<Item extends string>(plan: CatalogMaskPlan<Item>): void {
  const { protocol, catalog, expectedCount, expectedOrder } = plan
  if (expectedOrder.length !== expectedCount) {
    throw new Error(`${protocol} expected catalog must contain ${expectedCount} items; got ${expectedOrder.length}`)
  }
  if (catalog.length !== expectedCount) {
    throw new Error(`${protocol} catalog must contain ${expectedCount} items; got ${catalog.length}`)
  }
  if (new Set(catalog).size !== catalog.length) {
    throw new Error(`${protocol} catalog items must be unique`)
  }
  const mismatch = catalog.findIndex((item, index) => item !== expectedOrder[index])
  if (mismatch !== -1) {
    throw new Error(`${protocol} catalog item ${mismatch} must be ${expectedOrder[mismatch]}; got ${catalog[mismatch]}`)
  }
}

export function checkedCatalogMaskRanges<Item extends string>(
  plan: CatalogMaskPlan<Item>, enumerate: RangeEnumerator = enumerateRanges,
): CatalogMaskRange[] {
  assertCatalog(plan)
  const rangeSize = plan.rangeSize ?? DEFAULT_RANGE_SIZE
  if (!Number.isSafeInteger(rangeSize) || rangeSize <= 0) {
    throw new Error(`${plan.protocol} catalog mask range size must be a positive safe integer`)
  }
  return enumerate(2 ** plan.expectedCount, rangeSize)
}

export function catalogItemsForMask<Item extends string>(
  catalog: readonly Item[], mask: number,
): Item[] {
  return catalog.filter((_, index) => (mask & (1 << index)) !== 0)
}

function assertionDetail(value: unknown): string {
  const raw = value instanceof Error ? value.message : String(value)
  const compact = raw.replaceAll(/\s+/g, ' ').trim()
  return compact.slice(0, MAX_ASSERTION_DETAIL)
}

export function executeCatalogMaskRange<Item extends string>(
  protocol: string,
  catalog: readonly Item[],
  range: CatalogMaskRange,
  assertMask: (mask: number, enabledItems: readonly Item[]) => void,
): void {
  let attempted = 0
  let failed = 0
  let firstFailure: { mask: number; enabledItems: readonly Item[]; detail: string } | null = null
  for (let mask = range.start; mask < range.end; mask += 1) {
    const enabledItems = catalogItemsForMask(catalog, mask)
    attempted += 1
    try {
      assertMask(mask, enabledItems)
    } catch (error) {
      failed += 1
      if (!firstFailure) {
        firstFailure = { mask, enabledItems: [...enabledItems], detail: assertionDetail(error) }
      }
    }
  }
  if (!firstFailure) return
  const omitted = failed - 1
  throw new Error(
    `${protocol} catalog masks [${range.start}, ${range.end}) attempted=${attempted} failed=${failed}; `
    + `first mask=${firstFailure.mask} (0x${firstFailure.mask.toString(16)}) `
    + `enabled=${JSON.stringify(firstFailure.enabledItems)} assertion=${firstFailure.detail}; `
    + `omitted=${omitted}`,
  )
}
