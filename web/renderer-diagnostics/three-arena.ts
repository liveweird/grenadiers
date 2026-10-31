import { Vector3 } from 'three'
import type { ArenaRenderer } from '../src/game/arena/rendererTypes'
import { createThreeArenaRenderer } from '../src/game/arena/three/renderer'
import { createArenaCamera, fitArenaCamera } from '../src/game/arena/three/camera'
import { cellToWorld } from '../src/game/arena/three/coordinates'
import {
  threeArenaBloodLayerBaseline, threeArenaBloodLayerSnapshot,
  threeArenaDeathBaselineSnapshot, threeArenaDeathOverlapSnapshot, threeArenaDeathSnapshot,
  threeArenaSnapshot, type BloodLayerMode, type BoardName, type DeathOverlay, type SceneName,
} from './three-arena-snapshots'
import type { DeathAppearance } from '../src/game/arena/types'
import { mountColorCalibration, type CalibrationSamples } from './three-arena-calibration'
import { fireSnapshot, resetFireSnapshot, type FireReset, type FireScene } from './three-arena-fire'

type FrameCost = { samples: number; medianMs: number; p95Ms: number; maxMs: number }
type FixtureState = {
  board: BoardName
  scene: SceneName | null
  destroyed: boolean
  reducedMotion: boolean
  canvasCount: number
  canvasVisible: boolean
  pendingAnimationFrames: number
  contextLossSupported: boolean
}

export type ThreeArenaDiagnosticApi = {
  initialize(): Promise<FixtureState>
  show(board: BoardName, scene: SceneName): Promise<FixtureState>
  showDeath(board: BoardName, appearance: DeathAppearance, index: number): Promise<{
    state: FixtureState; bornAt: number
  }>
  showPoolOnly(board: BoardName, appearance: DeathAppearance, index: number): Promise<{
    state: FixtureState; bornAt: number
  }>
  replayDeath(
    board: BoardName,
    appearance: DeathAppearance,
    index: number,
    bornAt: number,
  ): Promise<FixtureState>
  resetDeathPresentation(board: BoardName, index: number): Promise<FixtureState>
  showBloodLayers(
    mode: BloodLayerMode,
    generation: number,
    bornAt: number | null,
  ): Promise<{ state: FixtureState; bornAt: number }>
  settleDeathAge(bornAt: number, ageMs: number): Promise<number>
  showDeathOverlap(
    overlay: DeathOverlay,
    includeDeath: boolean,
    generation: number,
    bornAt: number | null,
  ): Promise<{ state: FixtureState; bornAt: number }>
  suspend(): Promise<FixtureState>
  destroy(): Promise<FixtureState>
  readState(): FixtureState
  projectCell(board: BoardName, column: number, row: number): { x: number; y: number }
  loseAndRestoreContext(): Promise<{ supported: boolean; lost: FixtureState; restored: FixtureState }>
  calibrateColors(): Promise<CalibrationSamples>
  showFire(scene: FireScene): Promise<FixtureState>
  beginFireTransition(): Promise<{ state: FixtureState; bornAt: number }>
  settleFireAge(bornAt: number, ageMs: number): Promise<number>
  resetFire(reason: FireReset | 'preferences'): Promise<FixtureState>
  measureFireCost(scene: 'empty' | 'worstCase'): Promise<FrameCost>
  projectPoint(board: BoardName, column: number, row: number, elevation: number): { x: number; y: number }
}

declare global {
  interface Window { grenadiersThreeArena: ThreeArenaDiagnosticApi }
}

const nativeRequestAnimationFrame = window.requestAnimationFrame.bind(window)
const nativeCancelAnimationFrame = window.cancelAnimationFrame.bind(window)
const nativePerformanceNow = performance.now.bind(performance)
let fixtureNow: number | null = null
Object.defineProperty(performance, 'now', {
  configurable: true,
  value: (): number => fixtureNow ?? nativePerformanceNow(),
})
const pendingFrames = new Set<number>()
let measuredCallbackCosts: number[] | null = null
let insideCallback = false
let rescheduledInsideCallback = false
window.requestAnimationFrame = (callback: FrameRequestCallback): number => {
  if (insideCallback) rescheduledInsideCallback = true
  let handle = 0
  handle = nativeRequestAnimationFrame((timestamp) => {
    pendingFrames.delete(handle)
    insideCallback = true
    rescheduledInsideCallback = false
    const startedAt = nativePerformanceNow()
    try {
      callback(timestamp)
    } finally {
      const rendererLoop = rescheduledInsideCallback
      insideCallback = false
      if (rendererLoop) measuredCallbackCosts?.push(nativePerformanceNow() - startedAt)
    }
  })
  pendingFrames.add(handle)
  return handle
}
window.cancelAnimationFrame = (handle: number): void => {
  pendingFrames.delete(handle)
  nativeCancelAnimationFrame(handle)
}

function requireHost(): HTMLElement {
  const value = document.querySelector<HTMLElement>('#arena')
  if (!value) throw new Error('Three arena diagnostic host is unavailable')
  return value
}

const host = requireHost()
const calibrationHost = document.createElement('div')
calibrationHost.id = 'calibration'
calibrationHost.style.marginTop = '12px'
host.insertAdjacentElement('afterend', calibrationHost)

let renderer: ArenaRenderer | null = null
let board: BoardName = 'large'
let scene: SceneName | null = null
let destroyed = false
const reducedMotion = matchMedia('(prefers-reduced-motion: reduce)').matches
let disposeCalibration: (() => void) | null = null

function canvas(): HTMLCanvasElement | null {
  return host.querySelector('canvas')
}

function contextExtension(): WEBGL_lose_context | null {
  const target = canvas()
  if (!target) return null
  const gl = target.getContext('webgl2') ?? target.getContext('webgl')
  return gl?.getExtension('WEBGL_lose_context') ?? null
}

function readState(): FixtureState {
  const target = canvas()
  return {
    board, scene, destroyed, reducedMotion,
    canvasCount: host.querySelectorAll('canvas').length,
    canvasVisible: target !== null && getComputedStyle(target).visibility === 'visible',
    pendingAnimationFrames: pendingFrames.size,
    contextLossSupported: contextExtension() !== null,
  }
}

function waitFrame(): Promise<void> {
  return new Promise((resolve) => nativeRequestAnimationFrame(() => resolve()))
}

async function settle(frameCount = 3): Promise<void> {
  for (let index = 0; index < frameCount; index += 1) await waitFrame()
}

async function initialize(): Promise<FixtureState> {
  if (destroyed) throw new Error('Three arena diagnostic was destroyed')
  if (!renderer) {
    renderer = await createThreeArenaRenderer(host)
    renderer.setReducedMotion(reducedMotion)
  }
  return readState()
}

async function show(nextBoard: BoardName, nextScene: SceneName): Promise<FixtureState> {
  await initialize()
  board = nextBoard
  scene = nextScene
  renderer?.update(threeArenaSnapshot(nextBoard, nextScene))
  await settle()
  return readState()
}

async function showDeath(
  nextBoard: BoardName,
  appearance: DeathAppearance,
  index: number,
): Promise<{ state: FixtureState; bornAt: number }> {
  await initialize()
  board = nextBoard
  scene = null
  fixtureNow = nativePerformanceNow()
  const bornAt = fixtureNow
  renderer?.update(threeArenaDeathBaselineSnapshot(nextBoard, index))
  await settle(1)
  renderer?.update(threeArenaDeathSnapshot(nextBoard, appearance, bornAt, index))
  await settle(1)
  return { state: readState(), bornAt }
}

async function showPoolOnly(
  nextBoard: BoardName,
  appearance: DeathAppearance,
  index: number,
): Promise<{ state: FixtureState; bornAt: number }> {
  await initialize()
  board = nextBoard
  scene = null
  fixtureNow = nativePerformanceNow()
  const bornAt = fixtureNow - 1_000
  renderer?.update(threeArenaDeathBaselineSnapshot(nextBoard, index))
  await settle(1)
  renderer?.update(threeArenaDeathSnapshot(nextBoard, appearance, bornAt, index))
  await settle(2)
  return { state: readState(), bornAt }
}

async function replayDeath(
  nextBoard: BoardName,
  appearance: DeathAppearance,
  index: number,
  bornAt: number,
): Promise<FixtureState> {
  renderer?.update(threeArenaDeathSnapshot(nextBoard, appearance, bornAt, index))
  await settle(2)
  return readState()
}

async function resetDeathPresentation(nextBoard: BoardName, index: number): Promise<FixtureState> {
  renderer?.update(threeArenaDeathBaselineSnapshot(nextBoard, index))
  await settle(2)
  return readState()
}

async function showBloodLayers(
  mode: BloodLayerMode,
  generation: number,
  requestedBornAt: number | null,
): Promise<{ state: FixtureState; bornAt: number }> {
  await initialize()
  board = 'large'
  scene = null
  fixtureNow = requestedBornAt ?? nativePerformanceNow()
  const bornAt = requestedBornAt ?? fixtureNow - 1_000
  if (mode === 'poolOnly') {
    renderer?.update(threeArenaBloodLayerBaseline(generation))
    await settle(1)
  }
  renderer?.update(threeArenaBloodLayerSnapshot(mode, bornAt, generation))
  await settle(2)
  return { state: readState(), bornAt }
}

async function settleDeathAge(bornAt: number, ageMs: number): Promise<number> {
  fixtureNow = bornAt + ageMs
  await settle(2)
  return ageMs
}

async function showDeathOverlap(
  overlay: DeathOverlay,
  includeDeath: boolean,
  generation: number,
  requestedBornAt: number | null,
): Promise<{ state: FixtureState; bornAt: number }> {
  await initialize()
  board = 'large'
  scene = null
  const bornAt = requestedBornAt ?? nativePerformanceNow()
  fixtureNow = bornAt
  renderer?.update(threeArenaDeathOverlapSnapshot(overlay, includeDeath, bornAt, generation))
  await settle(2)
  return { state: readState(), bornAt }
}

async function suspend(): Promise<FixtureState> {
  renderer?.suspend()
  scene = null
  await new Promise((resolve) => setTimeout(resolve, 0))
  return readState()
}

async function destroy(): Promise<FixtureState> {
  renderer?.destroy()
  renderer = null
  scene = null
  destroyed = true
  await settle(2)
  return readState()
}

function projectCell(
  targetBoard: BoardName,
  column: number,
  row: number,
): { x: number; y: number } {
  return projectPoint(targetBoard, column, row, 0.4)
}

function projectPoint(
  targetBoard: BoardName,
  column: number,
  row: number,
  elevation: number,
): { x: number; y: number } {
  const dimensions = targetBoard === 'small' ? { width: 11, height: 9 }
    : targetBoard === 'medium' ? { width: 13, height: 11 } : { width: 15, height: 13 }
  const size = { width: host.clientWidth, height: host.clientHeight }
  const camera = createArenaCamera()
  fitArenaCamera(camera, dimensions, size)
  const [x, z] = cellToWorld(column, row)
  const projected = new Vector3(x, elevation, z).project(camera)
  return { x: (projected.x + 1) * size.width / 2, y: (1 - projected.y) * size.height / 2 }
}

async function loseAndRestoreContext(): Promise<{
  supported: boolean
  lost: FixtureState
  restored: FixtureState
}> {
  const extension = contextExtension()
  if (!extension) return { supported: false, lost: readState(), restored: readState() }
  extension.loseContext()
  await new Promise((resolve) => setTimeout(resolve, 50))
  const lost = readState()
  extension.restoreContext()
  await settle(5)
  return { supported: true, lost, restored: readState() }
}

async function calibrateColors(): Promise<CalibrationSamples> {
  disposeCalibration?.()
  const calibration = await mountColorCalibration(calibrationHost)
  disposeCalibration = calibration.dispose
  return calibration.samples
}

async function showFire(nextScene: FireScene): Promise<FixtureState> {
  await initialize()
  board = 'large'
  scene = null
  renderer?.update(fireSnapshot(nextScene))
  await settle()
  return readState()
}

async function beginFireTransition(): Promise<{ state: FixtureState; bornAt: number }> {
  await initialize()
  board = 'large'
  scene = null
  fixtureNow = Math.ceil(nativePerformanceNow() / 480) * 480
  renderer?.update(fireSnapshot('transitionBase', 10))
  await settle(1)
  fixtureNow += 1
  const bornAt = fixtureNow
  renderer?.update(fireSnapshot('transitionAdded', 11))
  await settle(2)
  return { state: readState(), bornAt }
}

async function settleFireAge(bornAt: number, ageMs: number): Promise<number> {
  return settleDeathAge(bornAt, ageMs)
}

async function resetFire(reason: FireReset | 'preferences'): Promise<FixtureState> {
  if (reason === 'preferences') {
    renderer?.setReducedMotion(!reducedMotion)
    renderer?.setReducedMotion(reducedMotion)
  }
  renderer?.update(resetFireSnapshot(reason === 'preferences' ? 'clear' : reason))
  await settle(2)
  return readState()
}

function percentile(sorted: readonly number[], fraction: number): number {
  return sorted[Math.min(sorted.length - 1, Math.floor(sorted.length * fraction))] ?? 0
}

async function measureFireCost(nextScene: 'empty' | 'worstCase'): Promise<FrameCost> {
  fixtureNow = null
  await showFire(nextScene)
  await settle(15)
  measuredCallbackCosts = []
  for (let attempts = 0; measuredCallbackCosts.length < 40 && attempts < 120; attempts += 1) {
    await waitFrame()
  }
  const sorted = measuredCallbackCosts.slice().sort((left, right) => left - right)
  measuredCallbackCosts = null
  return {
    samples: sorted.length,
    medianMs: percentile(sorted, 0.5),
    p95Ms: percentile(sorted, 0.95),
    maxMs: sorted.at(-1) ?? 0,
  }
}

window.addEventListener('pagehide', () => {
  renderer?.destroy()
  disposeCalibration?.()
}, { once: true })
window.grenadiersThreeArena = {
  initialize, show, showDeath, showPoolOnly, replayDeath, resetDeathPresentation,
  settleDeathAge, showDeathOverlap, showBloodLayers, suspend, destroy, readState, projectCell,
  loseAndRestoreContext, calibrateColors, showFire, beginFireTransition, settleFireAge,
  resetFire, measureFireCost, projectPoint,
}
