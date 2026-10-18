import { performance } from 'node:perf_hooks'
import { expect, type Page } from '@playwright/test'
import type {
  ArenaView, Cell, Direction, GameServerMessage, Position, Terrain,
} from '../../../web/src/game/v8/schema'
import type {
  DelayBridgeObserver, DelayBridgeSummary, RoutedFrameSummary, SanitizedSnapshot,
} from './delayed-websocket'

export type SanitizedArena = {
  width: number
  height: number
  terrain: Terrain[]
  blocks: Cell[]
}
type PageSnapshot = SanitizedSnapshot & { arena?: SanitizedArena }
type PageObservation =
  | { kind: 'installed' }
  | { kind: 'socket' }
  | { kind: 'movementSend'; timing: Omit<PageMovementSendTiming, 'endpoint'> }
  | { kind: 'snapshot'; snapshot: PageSnapshot; pageAt: number; pageTimeOrigin: number }

export type PageMovementSendTiming = {
  endpoint: string
  at: number
  timeOrigin: number
  direction: Direction | null
  sequence: number
}

const UNITS_PER_CELL = 1_024
const HALF_FOOTPRINT = 320
const MIN_FORWARD_CLEARANCE = 128
const MIN_MEASUREMENT_RUNWAY = UNITS_PER_CELL

export type PublicSnapshotTiming = {
  endpoint: string
  at: number
  revision: number
  tick: number
  roundId: number
  position: Position
  bombIds: number[]
  pageAt?: number
  pageTimeOrigin?: number
}

export type LatencySample = {
  kind: 'action' | 'release'
  direction: Direction | null
  keyStart: number
  keyFinished: number
  outgoingAt: number
  observedAt: number
  confirmedAt?: number
  latencyMs: number
}

export type TrajectoryChoice = {
  direction: 'east' | 'west'
  position: Position
  eastClearance: number
  westClearance: number
}

export class LatencyObserver implements DelayBridgeObserver {
  readonly frames: RoutedFrameSummary[] = []
  readonly forwardedFrames: RoutedFrameSummary[] = []
  readonly snapshots: PublicSnapshotTiming[] = []
  readonly pageMovementSends: PageMovementSendTiming[] = []
  readonly bridges: DelayBridgeSummary[] = []
  readonly failures: string[] = []
  readonly installedEndpoints = new Set<string>()
  readonly pageSocketCounts = new Map<string, number>()
  readonly arenas = new Map<string, SanitizedArena>()

  frame(summary: RoutedFrameSummary): void {
    this.frames.push(summary)
    if (this.frames.length > 2_048) this.frames.shift()
  }

  forwarded(summary: RoutedFrameSummary): void {
    this.forwardedFrames.push(summary)
    if (this.forwardedFrames.length > 2_048) this.forwardedFrames.shift()
  }

  bridge(summary: DelayBridgeSummary): void { this.bridges.push(summary) }
  failure(message: string): void { this.failures.push(message) }

  async observePageSnapshots(page: Page, endpoint: string): Promise<void> {
    const binding = '__grenadiersLatencySnapshot'
    await page.exposeBinding(binding, (_source, observation: PageObservation) => {
      if (observation.kind === 'installed') {
        this.installedEndpoints.add(endpoint)
        return
      }
      if (observation.kind === 'socket') {
        this.pageSocketCounts.set(endpoint, (this.pageSocketCounts.get(endpoint) ?? 0) + 1)
        return
      }
      if (observation.kind === 'movementSend') {
        this.pageMovementSends.push({ endpoint, ...observation.timing })
        if (this.pageMovementSends.length > 2_048) this.pageMovementSends.shift()
        return
      }
      const snapshot = observation.snapshot
      const { arena, ...publicSnapshot } = snapshot
      if (arena) this.arenas.set(endpoint, arena)
      const at = performance.now()
      const dispatch = this.forwardedFrames.find((frame) => frame.endpoint === endpoint
        && frame.direction === 'serverToClient' && frame.snapshot?.revision === snapshot.revision)
      if (!dispatch || at < dispatch.at) {
        this.failure(`Page snapshot ${endpoint}/${snapshot.revision} lacked a prior route dispatch`)
      }
      this.snapshots.push({
        endpoint, at, ...publicSnapshot,
        pageAt: observation.pageAt, pageTimeOrigin: observation.pageTimeOrigin,
      })
      if (this.snapshots.length > 2_048) this.snapshots.shift()
    })
    await page.addInitScript(({ bindingName }) => {
      const report = (value: PageObservation): void => {
        const binding = (window as unknown as Record<string, (item: PageObservation) => void>)[bindingName]
        binding(value)
      }
      const install = (): void => {
        if ((window as unknown as Record<string, unknown>).__grenadiersLatencyInstalled) return
        const NativeWebSocket = window.WebSocket
        let arenaSent = false
        class ObservedWebSocket extends NativeWebSocket {
          constructor(url: string | URL, protocols?: string | string[]) {
            if (protocols === undefined) super(url)
            else super(url, protocols)
            report({ kind: 'socket' })
            this.addEventListener('message', (event) => {
              const pageAt = performance.now()
              if (typeof event.data !== 'string') return
              try {
                const message = JSON.parse(event.data) as GameServerMessage
                if (message.type !== 'snapshot') return
                let actor: (typeof message.state.actors)[number] | undefined
                for (const candidate of message.state.actors) {
                  if (candidate.seat === 'one') actor = candidate
                }
                if (!actor || actor.type !== 'visible') return
                const bombIds: number[] = []
                for (const bomb of message.state.bombs) bombIds.push(bomb.id)
                let arena: ArenaView | undefined
                if (!arenaSent) {
                  const blocks: Cell[] = []
                  for (const cell of message.state.arena.blocks) blocks.push({ ...cell })
                  arena = {
                    width: message.state.arena.width, height: message.state.arena.height,
                    terrain: [...message.state.arena.terrain],
                    blocks,
                  }
                  arenaSent = true
                }
                report({ kind: 'snapshot', pageAt, pageTimeOrigin: performance.timeOrigin, snapshot: {
                  revision: message.room.revision, tick: message.room.tick, roundId: message.room.roundId,
                  position: actor.position, bombIds, arena,
                } })
              } catch { /* Non-snapshot frames remain private to the application. */ }
            })
          }

          override send(data: string | ArrayBuffer | Blob | ArrayBufferView<ArrayBuffer>): void {
            const at = performance.now()
            super.send(data)
            if (typeof data !== 'string') return
            try {
              const message = JSON.parse(data) as { type?: unknown; direction?: unknown; sequence?: unknown }
              const validDirection = message.direction === null
                || ['north', 'east', 'south', 'west'].includes(String(message.direction))
              if (message.type === 'movement' && validDirection && Number.isSafeInteger(message.sequence)) {
                report({ kind: 'movementSend', timing: {
                  at, timeOrigin: performance.timeOrigin,
                  direction: message.direction as Direction | null, sequence: message.sequence as number,
                } })
              }
            } catch { /* Only sanitized movement metadata is retained. */ }
          }
        }
        window.WebSocket = ObservedWebSocket
        ;(window as unknown as Record<string, unknown>).__grenadiersLatencyInstalled = true
        report({ kind: 'installed' })
      }
      document.addEventListener('readystatechange', () => {
        if (document.readyState === 'interactive') install()
      }, { once: true })
    }, { bindingName: binding })
  }

  async forwardedMovement(
    endpoint: string,
    afterIndex: number,
    direction: Direction | null,
  ): Promise<RoutedFrameSummary> {
    await expect.poll(() => this.forwardedFrames.slice(afterIndex).some((frame) => frame.endpoint === endpoint
      && frame.direction === 'clientToServer' && frame.type === 'movement'
      && frame.movement === direction), { intervals: [20] }).toBe(true)
    return this.forwardedFrames.slice(afterIndex).find((frame) => frame.endpoint === endpoint
      && frame.direction === 'clientToServer' && frame.type === 'movement'
      && frame.movement === direction)!
  }

  private ingressedAfter(endpoint: string, snapshot: PublicSnapshotTiming, after: number): boolean {
    return this.frames.some((frame) => frame.endpoint === endpoint && frame.direction === 'serverToClient'
      && frame.snapshot?.revision === snapshot.revision && frame.at >= after)
  }

  private hasForwardClearance(
    endpoint: string,
    snapshot: PublicSnapshotTiming,
    direction: 'east' | 'west',
  ): boolean {
    const arena = this.arenas.get(endpoint)
    return arena !== undefined && snapshot.bombIds.length === 0
      && horizontalClearance(arena, snapshot.position, direction) >= MIN_FORWARD_CLEARANCE
  }

  pageDeliveryHasPriorDispatch(endpoint: string, snapshot: PublicSnapshotTiming): boolean {
    return this.forwardedFrames.some((frame) => frame.endpoint === endpoint
      && frame.direction === 'serverToClient' && frame.snapshot?.revision === snapshot.revision
      && frame.at <= snapshot.at)
  }

  async twoMoved(
    endpoint: string,
    forwardedAt: number,
    baseline: PublicSnapshotTiming,
    direction: 'east' | 'west',
  ): Promise<[PublicSnapshotTiming, PublicSnapshotTiming]> {
    const pair = (): [PublicSnapshotTiming, PublicSnapshotTiming] | undefined => {
      const values = this.snapshots.filter((snapshot) => snapshot.endpoint === endpoint
        && snapshot.revision > baseline.revision && this.ingressedAfter(endpoint, snapshot, forwardedAt))
      for (let index = 1; index < values.length; index += 1) {
        const [previous, current] = [values[index - 1], values[index]]
        const progressing = direction === 'east'
          ? previous.position.x > baseline.position.x && current.position.x > previous.position.x
          : previous.position.x < baseline.position.x && current.position.x < previous.position.x
        if (progressing && previous.revision < current.revision
          && this.hasForwardClearance(endpoint, previous, direction)
          && this.hasForwardClearance(endpoint, current, direction)) return [previous, current]
      }
      return undefined
    }
    await expect.poll(() => pair() !== undefined, { intervals: [20] }).toBe(true)
    return pair()!
  }

  async firstStablePair(
    endpoint: string,
    forwardedAt: number,
    afterRevision: number,
    direction: 'east' | 'west',
  ): Promise<{
    observed: PublicSnapshotTiming
    confirmed: PublicSnapshotTiming
  }> {
    const pair = (): [PublicSnapshotTiming, PublicSnapshotTiming] | undefined => {
      const values = this.snapshots.filter((snapshot) => snapshot.endpoint === endpoint
        && snapshot.revision > afterRevision && this.ingressedAfter(endpoint, snapshot, forwardedAt))
      for (let index = 1; index < values.length; index += 1) {
        const previous = values[index - 1]
        const current = values[index]
        if (previous.revision < current.revision && previous.position.x === current.position.x
          && previous.position.y === current.position.y
          && this.hasForwardClearance(endpoint, previous, direction)
          && this.hasForwardClearance(endpoint, current, direction)) return [previous, current]
      }
      return undefined
    }
    await expect.poll(() => pair() !== undefined, { intervals: [20] }).toBe(true)
    const [observed, confirmed] = pair()!
    return { observed, confirmed }
  }

  latestSnapshot(endpoint: string): PublicSnapshotTiming {
    const snapshot = this.snapshots.filter((value) => value.endpoint === endpoint).at(-1)
    if (!snapshot) throw new Error(`No public snapshot observed for ${endpoint}`)
    return snapshot
  }

  safestHorizontalDirection(endpoint: string): TrajectoryChoice {
    const snapshot = this.latestSnapshot(endpoint)
    const arena = this.arenas.get(endpoint)
    if (!arena) throw new Error(`No public arena observed for ${endpoint}`)
    const eastClearance = horizontalClearance(arena, snapshot.position, 'east')
    const westClearance = horizontalClearance(arena, snapshot.position, 'west')
    const direction = eastClearance >= westClearance ? 'east' : 'west'
    if (Math.max(eastClearance, westClearance) < MIN_MEASUREMENT_RUNWAY) {
      throw new Error('No horizontal direction has one cell of collision-free measurement runway')
    }
    return { direction, position: snapshot.position, eastClearance, westClearance }
  }

  publicFactsAreMonotonic(endpoint: string): boolean {
    const values = this.snapshots.filter((snapshot) => snapshot.endpoint === endpoint)
    return values.every((current, index) => index === 0
      || (current.revision > values[index - 1].revision
        && current.tick >= values[index - 1].tick
        && current.roundId >= values[index - 1].roundId))
  }

  everyPageSnapshotFollowedRouteDispatch(endpoint: string): boolean {
    return this.snapshots.filter((snapshot) => snapshot.endpoint === endpoint)
      .every((snapshot) => this.pageDeliveryHasPriorDispatch(endpoint, snapshot))
  }
}

function horizontalClearance(
  arena: SanitizedArena,
  position: Position,
  direction: 'east' | 'west',
): number {
  let clearance = direction === 'east'
    ? arena.width * UNITS_PER_CELL - HALF_FOOTPRINT - position.x
    : position.x - HALF_FOOTPRINT
  const blocked: Cell[] = []
  for (let index = 0; index < arena.terrain.length; index += 1) {
    if (arena.terrain[index] === 'wall') {
      blocked.push({ column: index % arena.width, row: Math.floor(index / arena.width) })
    }
  }
  blocked.push(...arena.blocks)
  for (const cell of blocked) {
    const left = cell.column * UNITS_PER_CELL
    const right = left + UNITS_PER_CELL
    const top = cell.row * UNITS_PER_CELL
    const bottom = top + UNITS_PER_CELL
    if (position.y - HALF_FOOTPRINT >= bottom || position.y + HALF_FOOTPRINT <= top) continue
    if (direction === 'east' && position.x + HALF_FOOTPRINT <= left) {
      clearance = Math.min(clearance, left - HALF_FOOTPRINT - position.x)
    }
    if (direction === 'west' && position.x - HALF_FOOTPRINT >= right) {
      clearance = Math.min(clearance, position.x - HALF_FOOTPRINT - right)
    }
  }
  return Math.max(0, clearance)
}

export function distribution(samples: LatencySample[]) {
  const sorted = samples.map((sample) => sample.latencyMs).sort((left, right) => left - right)
  const value = (fraction: number): number => sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * fraction))]
  return {
    count: sorted.length,
    minMs: sorted[0],
    medianMs: value(0.5),
    observedP95Ms: value(0.95),
    maxMs: sorted.at(-1),
  }
}

export function sample(
  values: Omit<LatencySample, 'latencyMs'>,
): LatencySample {
  return {
    ...values,
    latencyMs: values.observedAt - values.keyStart,
  }
}
