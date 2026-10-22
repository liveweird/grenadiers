import { createHash } from 'node:crypto'
import { performance } from 'node:perf_hooks'

export type Cell = { column: number; row: number }
export type StageData = { physicalTick?: number; effectTicks?: number; deadline?: number;
  roundId?: number; bombId?: number; remainingTicks?: number }
export type StageReceipt = StageData & { stage: string; wall: string; monotonic: number;
  outcome?: 'started' | 'passed' | 'failed' }
export type LiveWindowDiagnostics = {
  stages: StageReceipt[]
  selectedBombs: { id: number; cell: Cell; placedAt: number; detonationDeadline: number }[]
  events: { type: 'bombPlaced' | 'bombDetonated'; bombId: number; tick: number; cell: Cell }[]
  originalEffectDeadline?: number
  secondEffectDeadline?: number
}
export type TaskOutcome<T> = { name: string; ok: true; value: T }
  | { name: string; ok: false; error: unknown }
export type PixelComparison = {
  name: string; leftSHA256: string; rightSHA256: string; passed: boolean
}

export function markStage(diagnostics: LiveWindowDiagnostics, stage: string,
  outcome?: StageReceipt['outcome'], data: StageData = {}): void {
  diagnostics.stages.push({ stage, wall: new Date().toISOString(),
    monotonic: performance.now(), outcome, ...data })
}

export function requireSuccessfulTasks(diagnostics: LiveWindowDiagnostics,
  outcomes: TaskOutcome<unknown>[]): void {
  const failures = outcomes.filter((outcome): outcome is Extract<TaskOutcome<unknown>, { ok: false }> => !outcome.ok)
  if (failures.length === 0) return
  markStage(diagnostics, 'live-window-phase', 'failed')
  throw new AggregateError(failures.map(({ error }) => error),
    `Live pixel window failed: ${failures.map(({ name }) => name).join(', ')}`)
}

export async function settleAll<T>(tasks: Promise<T>[]): Promise<T[]> {
  const outcomes = await Promise.allSettled(tasks)
  const failures = outcomes.filter((outcome): outcome is PromiseRejectedResult =>
    outcome.status === 'rejected')
  if (failures.length > 0) throw new AggregateError(failures.map(({ reason }) => reason))
  return outcomes.map((outcome) => (outcome as PromiseFulfilledResult<T>).value)
}

export function validateLiveGuard(input: {
  currentRoundId: number; expectedRoundId: number; physicalTick: number; effectTicks: number;
  effectDeadline: number; bombId: number;
  bomb?: { id: number; trigger: { type: string; remainingTicks?: number } }
}): void {
  if (input.currentRoundId !== input.expectedRoundId) throw new Error('live guard round changed')
  if (input.physicalTick + input.effectTicks !== input.effectDeadline) {
    throw new Error('live guard effect deadline changed')
  }
  if (!input.bomb || input.bomb.id !== input.bombId) throw new Error('live guard bomb missing')
  if (input.bomb.trigger.type !== 'timed'
    || (input.bomb.trigger.remainingTicks ?? 0) <= 0) throw new Error('live guard bomb expired')
}

export function recordPlacedBomb(diagnostics: LiveWindowDiagnostics,
  bomb: { id: number; cell: Cell; placedAt: number }, fuseTicks: number): void {
  diagnostics.selectedBombs.push({ id: bomb.id, cell: bomb.cell, placedAt: bomb.placedAt,
    detonationDeadline: bomb.placedAt + fuseTicks })
  diagnostics.events.push({ type: 'bombPlaced', bombId: bomb.id, tick: bomb.placedAt, cell: bomb.cell })
  markStage(diagnostics, 'bomb-placed', 'passed', { physicalTick: bomb.placedAt,
    bombId: bomb.id, deadline: bomb.placedAt + fuseTicks })
}

export function recordDetonatedBomb(diagnostics: LiveWindowDiagnostics,
  bomb: { id: number; cell: Cell; tick: number }, deadline: number): void {
  diagnostics.events.push({ type: 'bombDetonated', bombId: bomb.id, tick: bomb.tick, cell: bomb.cell })
  markStage(diagnostics, 'bomb-detonated', 'passed', {
    physicalTick: bomb.tick, bombId: bomb.id, deadline,
  })
}

export function recordPixelComparison(comparisons: PixelComparison[], name: string,
  left: string[], right: string[], equal: boolean): boolean {
  const digest = (value: string[]) => createHash('sha256').update(JSON.stringify(value)).digest('hex')
  const passed = equal === (JSON.stringify(left) === JSON.stringify(right))
  comparisons.push({ name, leftSHA256: digest(left), rightSHA256: digest(right), passed })
  return passed
}
