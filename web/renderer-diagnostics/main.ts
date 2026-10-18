import { createArenaRenderer, type ArenaRenderer } from '../src/game/arena/renderer'
import '../src/game/arena/arena.css'
import { diagnosticSnapshot } from './snapshot'

const UPDATE_INTERVAL_MS = 50
const WARMUP_DURATION_MS = 20_000
const TOTAL_DEADLINES = WARMUP_DURATION_MS / UPDATE_INTERVAL_MS
const TIMESTAMP_LIMIT = 128

type FinalMode = 'active' | 'suspended'
type LifecycleMode = 'empty' | FinalMode
type TimingSummary = {
  count: number
  elapsedMs: number
  missedDeadlines: number
  timestamps: number[]
  deliveredOrdinals: number[]
}
export type FixedRendererState = {
  buildTarget: 'grenadiers-renderer-diagnostic'
  mode: LifecycleMode
  initialized: boolean
  destroyed: boolean
  suspendCount: number
  reducedMotion: boolean
  warmHistory: 'deterministic-motion-20hz-20s-v1'
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

export type RendererDiagnosticApi = {
  initialize: () => Promise<void>
  warmup: () => Promise<void>
  finish: (mode: FinalMode) => void
  readFixedState: () => FixedRendererState
}

declare global {
  interface Window { grenadiersRendererDiagnostic: RendererDiagnosticApi }
}

function requireHost(): HTMLElement {
  const value = document.querySelector<HTMLElement>('#arena .game-arena__canvas')
  if (!value) throw new Error('Renderer diagnostic host is unavailable')
  return value
}

const host = requireHost()
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches

let renderer: ArenaRenderer | undefined
let initialCanvas: HTMLCanvasElement | undefined
let mode: LifecycleMode = 'empty'
let destroyed = false
let suspendCount = 0
let warmupPromise: Promise<void> | undefined
const updates: TimingSummary = {
  count: 0, elapsedMs: 0, missedDeadlines: 0, timestamps: [],
  deliveredOrdinals: [],
}
let finalMotionOrdinal: number | null = null
const raf = { count: 0, maxGapMs: 0, timestamps: [] as number[] }

function retain(values: number[], value: number): void {
  values.push(value)
  if (values.length > TIMESTAMP_LIMIT) values.shift()
}

let previousRaf: number | undefined
function observeRaf(timestamp: number): void {
  raf.count += 1
  if (previousRaf !== undefined) raf.maxGapMs = Math.max(raf.maxGapMs, timestamp - previousRaf)
  previousRaf = timestamp
  retain(raf.timestamps, timestamp)
  requestAnimationFrame(observeRaf)
}
requestAnimationFrame(observeRaf)

async function initialize(): Promise<void> {
  if (renderer) return
  if (destroyed) throw new Error('Renderer diagnostic cannot initialize after teardown')
  renderer = await createArenaRenderer(host)
  renderer.setReducedMotion(reducedMotion)
  initialCanvas = host.querySelector('canvas') ?? undefined
  if (!initialCanvas) throw new Error('Arena renderer did not attach a canvas')
}

function runWarmup(): Promise<void> {
  if (!renderer) throw new Error('Renderer diagnostic must be initialized before warmup')
  if (mode !== 'empty') throw new Error('Renderer diagnostic warmup already finished')
  const activeRenderer = renderer
  const startedAt = performance.now()
  let deadlineOrdinal = 1
  let lastMotionOrdinal = 1
  return new Promise((resolve, reject) => {
    const schedule = (): void => {
      const deadline = startedAt + deadlineOrdinal * UPDATE_INTERVAL_MS
      setTimeout(tick, Math.max(0, deadline - performance.now()))
    }
    const tick = (): void => {
      try {
        const now = performance.now()
        while (deadlineOrdinal < TOTAL_DEADLINES
          && now >= startedAt + (deadlineOrdinal + 1) * UPDATE_INTERVAL_MS) {
          updates.missedDeadlines += 1
          deadlineOrdinal += 1
        }
        if (deadlineOrdinal <= TOTAL_DEADLINES) {
          const motionOrdinal = deadlineOrdinal === TOTAL_DEADLINES
            ? lastMotionOrdinal : deadlineOrdinal
          activeRenderer.update(diagnosticSnapshot(deadlineOrdinal, motionOrdinal))
          lastMotionOrdinal = motionOrdinal
          updates.count += 1
          updates.deliveredOrdinals.push(deadlineOrdinal)
          finalMotionOrdinal = motionOrdinal
          retain(updates.timestamps, now)
          deadlineOrdinal += 1
        }
        if (deadlineOrdinal > TOTAL_DEADLINES) {
          updates.elapsedMs = performance.now() - startedAt
          resolve()
        } else schedule()
      } catch (error) {
        reject(error)
      }
    }
    schedule()
  })
}

async function warmup(): Promise<void> {
  warmupPromise ??= runWarmup()
  await warmupPromise
}

function finish(nextMode: FinalMode): void {
  if (!renderer || !warmupPromise || updates.elapsedMs === 0) {
    throw new Error('Renderer diagnostic cannot finish before warmup completes')
  }
  if (mode !== 'empty') throw new Error('Renderer diagnostic finish was already called')
  if (nextMode === 'suspended') {
    renderer.suspend()
    suspendCount += 1
  }
  mode = nextMode
}

function readFixedState(): FixedRendererState {
  const canvases = [...host.querySelectorAll('canvas')]
  const current = canvases[0]
  return {
    buildTarget: 'grenadiers-renderer-diagnostic', mode,
    initialized: renderer !== undefined, destroyed, suspendCount, reducedMotion,
    warmHistory: 'deterministic-motion-20hz-20s-v1',
    updates: {
      ...updates,
      timestamps: [...updates.timestamps],
      deliveredOrdinals: [...updates.deliveredOrdinals],
    },
    finalMotionOrdinal,
    raf: { ...raf, timestamps: [...raf.timestamps] },
    viewport: { width: innerWidth, height: innerHeight },
    document: {
      width: document.documentElement.scrollWidth,
      height: document.documentElement.scrollHeight,
    },
    canvas: {
      count: canvases.length,
      identity: initialCanvas ? 1 : null,
      stable: initialCanvas === undefined ? canvases.length === 0 : current === initialCanvas,
      connected: initialCanvas?.isConnected ?? false,
      visibility: initialCanvas ? getComputedStyle(initialCanvas).visibility : null,
    },
    capturedAt: performance.now(), timeOrigin: performance.timeOrigin,
  }
}

window.addEventListener('pagehide', () => {
  if (!renderer || destroyed) return
  renderer.destroy()
  destroyed = true
}, { once: true })

window.grenadiersRendererDiagnostic = { initialize, warmup, finish, readFixedState }
