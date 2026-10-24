import {
  assertValidRuntimeManifest, assertValidSourceManifest,
  buildRuntimeManifest, createSourceManifest, type ArtManifest, type PixelBuffer,
  type RuntimeArtManifest,
} from '../src/game/arena/artAtlas'
import { bindArtAtlasControls } from './art-atlas-controls'
import {
  compareImportedFrame as compareFrame,
  createEditedTemplate as editTemplate,
  type PixelEdit,
} from './art-atlas-diagnostics'
import { artAtlasText, type ArtAtlasLocale } from './art-atlas-locales'
import {
  base64ToBytes, bytesToBase64, decodePng, packRuntimePixels, pixelCanvas, pngBytes, slicePixels,
} from './art-atlas-pixels'
import {
  guideSvg, renderBaselineSource, renderGuides, renderOverview,
  readPreviewResourceState, renderRuntimePreview, type PreviewController,
} from './art-atlas-render'

type OutputFile = { name: string; mime: string; base64?: string; text?: string }
type ExportRequest = { pngBase64: string; manifestJson: string; slices?: boolean }
type ExportResult = {
  files: OutputFile[]
  runtimeManifest: RuntimeArtManifest
  rgbaBytes: number
  renderMs: number
  spriteCount: number
}
type DiagnosticState = {
  initialized: boolean
  rebuildCount: number
  canvasCount: number
  spriteCount: number
  frameCount: number
  texture: { width: number; height: number; rgbaBytes: number } | null
  renderMs: number | null
  resources: { created: number; destroyed: number; live: number }
}
type StatusKey = 'ready' | 'validating' | 'valid' | 'exported'
type RuntimeBuild = {
  manifest: RuntimeArtManifest
  pixels: PixelBuffer
  preview: PreviewController
}
export type ArtAtlasDiagnosticApi = {
  initialize(): Promise<void>
  createTemplate(): Promise<{ files: OutputFile[]; manifest: ArtManifest }>
  exportSource(request: ExportRequest): Promise<ExportResult>
  resetPreview(): Promise<DiagnosticState>
  restorePreviewContext(): Promise<DiagnosticState>
  measureRenders(count?: number): { count: number; elapsedMs: number }
  createEditedTemplate(edit: PixelEdit): Promise<{ pngBase64: string; manifestJson: string }>
  compareImportedFrame(id: string): Promise<{
    differentBytes: number
    bounds: { x: number; y: number; width: number; height: number }
  }>
  readState(): DiagnosticState
}

declare global {
  interface Window { grenadiersArtAtlas: ArtAtlasDiagnosticApi }
}

const sourceCanvas = requireElement<HTMLCanvasElement>('source-canvas')
const guideCanvas = requireElement<HTMLCanvasElement>('guide-canvas')
const previewHost = requireElement<HTMLElement>('preview-host')
const sheetStage = requireElement<HTMLElement>('sheet-stage')
const status = requireElement<HTMLElement>('status')
const selection = requireElement<HTMLElement>('selection-detail')
const inventory = requireElement<HTMLOListElement>('inventory')
const measurement = requireElement<HTMLOutputElement>('measurement')

let locale: ArtAtlasLocale = 'en'
let currentManifest: ArtManifest | undefined
let currentSource: PixelBuffer | undefined
let currentRuntime: PixelBuffer | undefined
let currentRuntimeManifest: RuntimeArtManifest | undefined
let preview: PreviewController | undefined
let rebuildCount = 0
let initializePromise: Promise<void> | undefined
let selectedFrameId: string | undefined
let statusState: { key: StatusKey; error?: string } = { key: 'ready' }
let mutationQueue: Promise<void> = Promise.resolve()

function requireElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id)
  if (!element) throw new Error(`Missing art atlas element #${id}`)
  return element as T
}

function canonical(value: unknown): unknown {
  if (Array.isArray(value)) return value.map(canonical)
  if (value && typeof value === 'object') {
    return Object.fromEntries(Object.entries(value).sort(([left], [right]) => left.localeCompare(right))
      .map(([key, entry]) => [key, canonical(entry)]))
  }
  return value
}

function canonicalJson(value: unknown): string {
  return `${JSON.stringify(canonical(value), null, 2)}\n`
}

function serializeMutation<T>(operation: () => Promise<T>): Promise<T> {
  const result = mutationQueue.then(operation, operation)
  mutationQueue = result.then(() => undefined, () => undefined)
  return result
}

function parseManifest(text: string): ArtManifest {
  if (new TextEncoder().encode(text).length > 1024 * 1024) throw new Error('Manifest exceeds the 1MiB input limit')
  const value: unknown = JSON.parse(text)
  if (!value || typeof value !== 'object' || !('sheet' in value) || !('frames' in value)) {
    throw new Error('Manifest must be an art atlas JSON object')
  }
  const manifest = value as ArtManifest
  assertValidSourceManifest(manifest)
  return manifest
}

async function fileFromPixels(name: string, pixels: PixelBuffer): Promise<OutputFile> {
  return { name, mime: 'image/png', base64: bytesToBase64(await pngBytes(pixels)) }
}

function fileFromText(name: string, mime: string, text: string): OutputFile {
  return { name, mime, text }
}

async function templateFiles(manifest: ArtManifest, pixels: PixelBuffer): Promise<OutputFile[]> {
  return Promise.all([
    fileFromPixels('arena-source.png', pixels),
    Promise.resolve(fileFromText('arena-source.json', 'application/json', canonicalJson(manifest))),
    Promise.resolve(fileFromText('arena-guides.svg', 'image/svg+xml', guideSvg(manifest))),
    fileFromPixels('arena-guides.png', renderGuides(manifest)),
    fileFromPixels('arena-overview.png', renderOverview(pixels, manifest)),
  ])
}

function updateCanvas(target: HTMLCanvasElement, pixels: PixelBuffer): void {
  target.width = pixels.width
  target.height = pixels.height
  const context = target.getContext('2d')
  if (!context) throw new Error('Canvas 2D is unavailable')
  context.clearRect(0, 0, target.width, target.height)
  context.drawImage(pixelCanvas(pixels), 0, 0)
}

function updateSheetScale(value: number): void {
  if (!currentSource) return
  const scale = value / 100
  const width = `${currentSource.width * scale}px`
  const height = `${currentSource.height * scale}px`
  sheetStage.style.width = width
  sheetStage.style.height = height
  for (const canvas of [sourceCanvas, guideCanvas]) {
    canvas.style.width = width
    canvas.style.height = height
  }
  requireElement<HTMLOutputElement>('zoom-value').value = `${value}%`
}

function populateInventory(manifest: ArtManifest): void {
  const text = artAtlasText(locale)
  inventory.replaceChildren(...manifest.frames.map((frame) => {
    const item = document.createElement('li')
    const button = document.createElement('button')
    button.type = 'button'
    const statusText = frame.status === 'reserved' ? `  ${text.reservedStatus}` : ''
    button.textContent = `${frame.id}  ${frame.sourceRect.width}×${frame.sourceRect.height}${statusText}`
    button.addEventListener('click', () => selectFrame(frame.id))
    item.appendChild(button)
    return item
  }))
}

function selectFrame(id: string): void {
  const frame = currentManifest?.frames.find((candidate) => candidate.id === id)
  if (!frame) return
  selectedFrameId = id
  const text = artAtlasText(locale)
  const statusText = frame.status === 'reserved' ? text.reservedStatus : text.baselineStatus
  selection.textContent = `${frame.id}\n${frame.sourceRect.width} × ${frame.sourceRect.height}px · pivot ${frame.pivot.x},${frame.pivot.y} · ${statusText}`
  const scale = Number(requireElement<HTMLInputElement>('zoom').value) / 100
  requireElement<HTMLElement>('sheet-scroll').scrollTo({
    left: Math.max(0, frame.sourceRect.x * scale - 24),
    top: Math.max(0, frame.sourceRect.y * scale - 24),
    behavior: matchMedia('(prefers-reduced-motion: reduce)').matches ? 'auto' : 'smooth',
  })
}

function renderSemanticState(): void {
  status.textContent = statusState.error ?? artAtlasText(locale)[statusState.key]
  if (selectedFrameId) selectFrame(selectedFrameId)
  else selection.textContent = artAtlasText(locale).noSelection
}

function setStatus(key: StatusKey, error?: string): void {
  statusState = { key, ...(error ? { error } : {}) }
  renderSemanticState()
}

function selectAt(clientX: number, clientY: number): void {
  if (!currentManifest) return
  const bounds = sourceCanvas.getBoundingClientRect()
  const x = (clientX - bounds.left) * sourceCanvas.width / bounds.width
  const y = (clientY - bounds.top) * sourceCanvas.height / bounds.height
  const frame = currentManifest.frames.find(({ sourceRect }) => x >= sourceRect.x && y >= sourceRect.y
    && x < sourceRect.x + sourceRect.width && y < sourceRect.y + sourceRect.height)
  if (frame) selectFrame(frame.id)
}

function showSource(pixels: PixelBuffer, manifest: ArtManifest): void {
  currentSource = pixels
  currentManifest = manifest
  updateCanvas(sourceCanvas, pixels)
  updateCanvas(guideCanvas, renderGuides(manifest))
  updateSheetScale(Number(requireElement<HTMLInputElement>('zoom').value))
  populateInventory(manifest)
}

async function replacePreview(pixels: PixelBuffer, manifest: RuntimeArtManifest): Promise<PreviewController> {
  const next = await renderRuntimePreview(pixels, manifest)
  const previous = preview
  previewHost.replaceChildren(next.app.canvas)
  preview = next
  currentRuntime = pixels
  currentRuntimeManifest = manifest
  rebuildCount += 1
  measurement.value = `${next.spriteCount} ${artAtlasText(locale).sprites} · ${next.renderMs.toFixed(2)} ms`
  previous?.destroy()
  return next
}

async function buildFromSource(pixels: PixelBuffer, manifest: ArtManifest): Promise<RuntimeBuild> {
  assertValidSourceManifest(manifest, pixels)
  const runtimeManifest = buildRuntimeManifest(manifest)
  const runtimePixels = packRuntimePixels(pixels, manifest, runtimeManifest)
  assertValidRuntimeManifest(runtimeManifest, runtimePixels)
  const nextPreview = await replacePreview(runtimePixels, runtimeManifest)
  return { manifest: runtimeManifest, pixels: runtimePixels, preview: nextPreview }
}

async function initialize(): Promise<void> {
  initializePromise ??= serializeMutation(async () => {
    const manifest = createSourceManifest()
    const pixels = await renderBaselineSource(manifest)
    showSource(pixels, manifest)
    await buildFromSource(pixels, manifest)
    setStatus('ready')
  })
  await initializePromise
}

async function createTemplate(): Promise<{ files: OutputFile[]; manifest: ArtManifest }> {
  const manifest = createSourceManifest()
  const pixels = await renderBaselineSource(manifest)
  assertValidSourceManifest(manifest, pixels)
  return { files: await templateFiles(manifest, pixels), manifest }
}

async function exportSourceInternal(request: ExportRequest): Promise<ExportResult> {
  if (request.pngBase64.length > 44 * 1024 * 1024) {
    throw new Error('Compressed source PNG exceeds the 32MiB input limit')
  }
  const manifest = parseManifest(request.manifestJson)
  const source = await decodePng(base64ToBytes(request.pngBase64), manifest)
  const runtime = await buildFromSource(source, manifest)
  const files: OutputFile[] = [
    await fileFromPixels('arena-atlas.png', runtime.pixels),
    fileFromText('arena-atlas.json', 'application/json', canonicalJson(runtime.manifest)),
    await fileFromPixels('arena-overview.png', renderOverview(source, manifest)),
    await fileFromPixels('arena-preview.png', runtime.preview.pixels),
  ]
  if (request.slices) {
    for (const frame of runtime.manifest.frames) {
      files.push(await fileFromPixels(`slices/${frame.id}.png`, slicePixels(
        runtime.pixels, frame.sourceRect.x, frame.sourceRect.y,
        frame.sourceRect.width, frame.sourceRect.height,
      )))
    }
  }
  showSource(source, manifest)
  return {
    files, runtimeManifest: runtime.manifest, rgbaBytes: runtime.pixels.data.length,
    renderMs: runtime.preview.renderMs, spriteCount: runtime.preview.spriteCount,
  }
}

function exportSource(request: ExportRequest): Promise<ExportResult> {
  return serializeMutation(() => exportSourceInternal(request))
}

function readState(): DiagnosticState {
  return {
    initialized: currentManifest !== undefined && preview !== undefined,
    rebuildCount,
    canvasCount: previewHost.querySelectorAll('canvas').length,
    spriteCount: preview?.spriteCount ?? 0,
    frameCount: currentManifest?.frames.length ?? 0,
    texture: currentRuntime ? {
      width: currentRuntime.width, height: currentRuntime.height, rgbaBytes: currentRuntime.data.length,
    } : null,
    renderMs: preview?.renderMs ?? null,
    resources: readPreviewResourceState(),
  }
}

async function resetPreviewInternal(): Promise<DiagnosticState> {
  const manifest = createSourceManifest()
  const pixels = await renderBaselineSource(manifest)
  showSource(pixels, manifest)
  await buildFromSource(pixels, manifest)
  return readState()
}

function resetPreview(): Promise<DiagnosticState> {
  return serializeMutation(resetPreviewInternal)
}

function eventOnce(target: EventTarget, name: string): Promise<void> {
  return new Promise((resolve) => target.addEventListener(name, () => resolve(), { once: true }))
}

async function restorePreviewContextInternal(): Promise<DiagnosticState> {
  if (!preview || !('context' in preview.app.renderer)) throw new Error('Preview renderer is not WebGL')
  const lost = eventOnce(preview.app.canvas, 'webglcontextlost')
  const restored = eventOnce(preview.app.canvas, 'webglcontextrestored')
  preview.app.renderer.context.forceContextLoss()
  await lost
  await restored
  preview.app.render()
  return readState()
}

function restorePreviewContext(): Promise<DiagnosticState> {
  return serializeMutation(restorePreviewContextInternal)
}

function measureRenders(count = 20): { count: number; elapsedMs: number } {
  if (!preview) throw new Error('Preview is not initialized')
  if (!Number.isInteger(count) || count < 1 || count > 120) throw new RangeError('Render count must be from 1 through 120')
  const started = performance.now()
  for (let index = 0; index < count; index += 1) preview.app.render()
  return { count, elapsedMs: performance.now() - started }
}

async function createEditedTemplate(edit: PixelEdit): Promise<{ pngBase64: string; manifestJson: string }> {
  return editTemplate(edit, canonicalJson)
}

async function compareImportedFrame(id: string): Promise<{
  differentBytes: number
  bounds: { x: number; y: number; width: number; height: number }
}> {
  return compareFrame(id, {
    source: currentSource, sourceManifest: currentManifest,
    runtime: currentRuntime, runtimeManifest: currentRuntimeManifest,
  })
}

function applyLocale(next: ArtAtlasLocale): void {
  locale = next
  const text = artAtlasText(locale) as unknown as Record<string, string>
  document.documentElement.lang = locale
  document.title = text.documentTitle
  document.querySelectorAll<HTMLElement>('[data-text]').forEach((element) => {
    const key = element.dataset.text
    if (key && text[key]) element.textContent = text[key]
  })
  document.querySelectorAll<HTMLElement>('[data-text-aria]').forEach((element) => {
    const key = element.dataset.textAria
    if (key && text[key]) element.setAttribute('aria-label', text[key])
  })
  if (currentManifest) populateInventory(currentManifest)
  if (preview) measurement.value = `${preview.spriteCount} ${artAtlasText(locale).sprites} · ${preview.renderMs.toFixed(2)} ms`
  renderSemanticState()
}

bindArtAtlasControls({
  sourceCanvas, sheetStage, setStatus,
  applyLocale, updateSheetScale, selectAt, resetPreview, exportSource, canonicalJson,
  currentSource: () => ({ manifest: currentManifest, pixels: currentSource }),
})
applyLocale('en')
window.grenadiersArtAtlas = {
  initialize, createTemplate, exportSource, resetPreview,
  restorePreviewContext, measureRenders, createEditedTemplate, compareImportedFrame, readState,
}
if (new URL(location.href).searchParams.get('cli') !== '1') void initialize()
