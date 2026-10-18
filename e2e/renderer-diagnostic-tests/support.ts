import { expect, type BrowserContext, type Page, type TestInfo } from '@playwright/test'

export type CellState = 'E' | 'A' | 'S'
export type MatrixCell = { name: string; primary: CellState; peer: CellState }
type TimingSummary = {
  count: number
  elapsedMs: number
  missedDeadlines: number
  timestamps: number[]
  deliveredOrdinals: number[]
}
export type FixedRendererState = {
  buildTarget: string
  mode: 'empty' | 'active' | 'suspended'
  initialized: boolean
  destroyed: boolean
  suspendCount: number
  reducedMotion: boolean
  warmHistory: string
  updates: TimingSummary
  finalMotionOrdinal: number | null
  raf: { count: number; maxGapMs: number; timestamps: number[] }
  viewport: { width: number; height: number }
  document: { width: number; height: number }
  canvas: {
    count: number
    identity: number | null
    stable: boolean
    connected: boolean
    visibility: string | null
  }
  capturedAt: number
  timeOrigin: number
}

type DiagnosticWindow = Window & {
  grenadiersRendererDiagnostic?: {
    initialize: () => Promise<void>
    warmup: () => Promise<void>
    finish: (mode: 'active' | 'suspended') => void
    readFixedState: () => FixedRendererState
  }
}

export type RequestGuard = {
  unexpected: Array<{ kind: 'cross-origin' | 'websocket'; resourceType: string }>
  installPageChecks: (page: Page) => void
}
export type LifecycleEvidence = {
  preCapture: Partial<Record<'primary' | 'peer', FixedRendererState>>
  postCapture: Partial<Record<'primary' | 'peer', FixedRendererState>>
}
export type LifecycleTimestamp = {
  stage: 'started' | 'navigated' | 'initialize-completed' | 'warmup-completed'
    | 'finish-applied' | 'capture-completed'
  nodeAt: number
}

const CELL_PATTERN = /^renderer-([EAS])-([EAS])$/

export function matrixCell(projectName: string): MatrixCell {
  const match = CELL_PATTERN.exec(projectName)
  if (!match) throw new Error(`Unexpected renderer diagnostic project: ${projectName}`)
  return { name: `${match[1]}/${match[2]}`, primary: match[1] as CellState, peer: match[2] as CellState }
}

export async function guardStaticOrigin(
  context: BrowserContext,
  baseUrl: string,
): Promise<RequestGuard> {
  const expectedOrigin = new URL(baseUrl).origin
  const unexpected: RequestGuard['unexpected'] = []
  await context.route('**/*', async (route) => {
    const request = route.request()
    const origin = new URL(request.url()).origin
    if (origin === expectedOrigin) await route.continue()
    else {
      unexpected.push({ kind: 'cross-origin', resourceType: request.resourceType() })
      await route.abort('blockedbyclient')
    }
  })
  return {
    unexpected,
    installPageChecks(page) {
      page.on('websocket', () => {
        unexpected.push({ kind: 'websocket', resourceType: 'websocket' })
      })
    },
  }
}

export async function initializeLifecycle(page: Page, state: CellState): Promise<void> {
  if (state === 'E') return
  await page.evaluate(async () => {
    const api = (window as DiagnosticWindow).grenadiersRendererDiagnostic
    if (!api) throw new Error('Renderer diagnostic API is unavailable')
    await api.initialize()
  })
}

export async function warmLifecycle(page: Page, state: CellState): Promise<void> {
  if (state === 'E') return
  await page.evaluate(async () => {
    const api = (window as DiagnosticWindow).grenadiersRendererDiagnostic
    if (!api) throw new Error('Renderer diagnostic API is unavailable')
    await api.warmup()
  })
}

export async function finishLifecycle(page: Page, state: CellState): Promise<void> {
  if (state === 'E') return
  await page.evaluate((requestedState) => {
    const api = (window as DiagnosticWindow).grenadiersRendererDiagnostic
    if (!api) throw new Error('Renderer diagnostic API is unavailable')
    api.finish(requestedState === 'A' ? 'active' : 'suspended')
  }, state)
}

export async function readFixedState(page: Page): Promise<FixedRendererState> {
  return page.evaluate(() => {
    const api = (window as DiagnosticWindow).grenadiersRendererDiagnostic
    if (!api) throw new Error('Renderer diagnostic API is unavailable')
    return api.readFixedState()
  })
}

function expectOrderedBounded(values: number[]): void {
  expect(values.length).toBeLessThanOrEqual(128)
  for (let index = 1; index < values.length; index += 1) {
    expect(values[index]).toBeGreaterThanOrEqual(values[index - 1])
  }
}

export function assertFixedState(
  actual: FixedRendererState,
  expected: CellState,
  role: 'primary' | 'peer',
): void {
  const viewport = role === 'primary' ? { width: 1440, height: 1050 } : { width: 320, height: 844 }
  const documentHeight = role === 'primary' ? 1334 : 1842
  expect(actual.buildTarget).toBe('grenadiers-renderer-diagnostic')
  expect(actual.viewport).toEqual(viewport)
  expect(actual.document).toEqual({ width: viewport.width, height: documentHeight })
  expect(actual.reducedMotion).toBe(role === 'peer')
  expect(actual.destroyed).toBe(false)
  expect(actual.raf.count).toBeGreaterThan(0)
  expect(actual.raf.maxGapMs).toBeGreaterThanOrEqual(0)
  expect(actual.timeOrigin).toBeGreaterThan(0)
  expect(actual.capturedAt).toBeGreaterThanOrEqual(0)
  expectOrderedBounded(actual.raf.timestamps)
  expectOrderedBounded(actual.updates.timestamps)
  if (expected === 'E') {
    expect(actual).toMatchObject({
      mode: 'empty', initialized: false, suspendCount: 0,
      updates: {
        count: 0, elapsedMs: 0, missedDeadlines: 0,
        deliveredOrdinals: [],
      },
      finalMotionOrdinal: null,
      canvas: {
        count: 0, identity: null, stable: true, connected: false, visibility: null,
      },
    })
    return
  }
  expect(actual.warmHistory).toBe('deterministic-motion-20hz-20s-v1')
  expect(actual.updates.count + actual.updates.missedDeadlines).toBe(400)
  expect(actual.updates.missedDeadlines).toBe(400 - actual.updates.count)
  expect(actual.updates.count).toBeGreaterThan(0)
  expect(actual.updates.count).toBe(actual.updates.deliveredOrdinals.length)
  expect(actual.updates.deliveredOrdinals.at(-1)).toBe(400)
  for (const [index, ordinal] of actual.updates.deliveredOrdinals.entries()) {
    expect(ordinal).toBeGreaterThanOrEqual(1)
    expect(ordinal).toBeLessThanOrEqual(400)
    if (index > 0) expect(ordinal).toBeGreaterThan(actual.updates.deliveredOrdinals[index - 1])
  }
  expect(actual.finalMotionOrdinal)
    .toBe(actual.updates.deliveredOrdinals.at(-2) ?? 1)
  expect(actual.updates.elapsedMs).toBeGreaterThanOrEqual(19_900)
  expect(actual.canvas).toMatchObject({ count: 1, identity: 1, stable: true, connected: true })
  expect(actual.mode).toBe(expected === 'A' ? 'active' : 'suspended')
  expect(actual.suspendCount).toBe(expected === 'S' ? 1 : 0)
  expect(actual.canvas.visibility).toBe(expected === 'S' ? 'hidden' : 'visible')
}

export function assertCapturePreservedState(
  before: FixedRendererState,
  after: FixedRendererState,
): void {
  expect(after.updates).toEqual(before.updates)
  expect(after).toMatchObject({
    mode: before.mode,
    initialized: before.initialized,
    destroyed: before.destroyed,
    suspendCount: before.suspendCount,
    reducedMotion: before.reducedMotion,
    warmHistory: before.warmHistory,
    finalMotionOrdinal: before.finalMotionOrdinal,
    viewport: before.viewport,
    document: before.document,
    canvas: before.canvas,
  })
}

export async function attachLifecycle(
  testInfo: TestInfo,
  cell: MatrixCell,
  timestamps: LifecycleTimestamp[],
  states: LifecycleEvidence,
  unexpectedRequests: RequestGuard['unexpected'],
): Promise<void> {
  await testInfo.attach('renderer-lifecycle', {
    contentType: 'application/json',
    body: JSON.stringify({ cell, timestamps: timestamps.slice(-128), states, unexpectedRequests }),
  })
}
