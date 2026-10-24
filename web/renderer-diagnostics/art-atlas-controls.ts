import { ArtAtlasValidationError, type ArtManifest, type PixelBuffer } from '../src/game/arena/artAtlas'
import type { ArtAtlasLocale } from './art-atlas-locales'
import { base64ToBytes, bytesToBase64, pngBytes } from './art-atlas-pixels'

type OutputFile = { name: string; mime: string; base64?: string; text?: string }
type Bindings = {
  sourceCanvas: HTMLCanvasElement
  sheetStage: HTMLElement
  setStatus(key: 'ready' | 'validating' | 'valid' | 'exported', error?: string): void
  applyLocale(locale: ArtAtlasLocale): void
  updateSheetScale(value: number): void
  selectAt(clientX: number, clientY: number): void
  resetPreview(): Promise<unknown>
  exportSource(request: { pngBase64: string; manifestJson: string }): Promise<{ files: OutputFile[] }>
  currentSource(): { manifest?: ArtManifest; pixels?: PixelBuffer }
  canonicalJson(value: unknown): string
}

function requireElement<T extends HTMLElement>(id: string): T {
  const element = document.getElementById(id)
  if (!element) throw new Error(`Missing art atlas element #${id}`)
  return element as T
}

function download(file: OutputFile): void {
  const contents = file.text ?? base64ToBytes(file.base64 ?? '')
  const part = typeof contents === 'string' ? contents : new Uint8Array(contents).buffer
  const blob = new Blob([part], { type: file.mime })
  const anchor = document.createElement('a')
  anchor.href = URL.createObjectURL(blob)
  anchor.download = file.name.replace('slices/', '')
  anchor.click()
  setTimeout(() => URL.revokeObjectURL(anchor.href), 0)
}

async function loadFiles(bindings: Bindings): Promise<void> {
  bindings.setStatus('validating')
  try {
    const png = requireElement<HTMLInputElement>('source-file').files?.[0]
    const json = requireElement<HTMLInputElement>('manifest-file').files?.[0]
    if (!png || !json) throw new Error('Choose both the source PNG and source manifest')
    if (png.size > 32 * 1024 * 1024) throw new Error('Compressed source PNG exceeds the 32MiB input limit')
    if (json.size > 1024 * 1024) throw new Error('Source manifest exceeds the 1MiB input limit')
    await bindings.exportSource({
      pngBase64: bytesToBase64(new Uint8Array(await png.arrayBuffer())),
      manifestJson: await json.text(),
    })
    bindings.setStatus('valid')
  } catch (error) {
    bindings.setStatus('validating', error instanceof ArtAtlasValidationError ? error.message : String(error))
  }
}

async function exportCurrent(bindings: Bindings): Promise<void> {
  const { manifest, pixels } = bindings.currentSource()
  if (!manifest || !pixels) return
  const result = await bindings.exportSource({
    pngBase64: bytesToBase64(await pngBytes(pixels)),
    manifestJson: bindings.canonicalJson(manifest),
  })
  result.files.forEach(download)
  bindings.setStatus('exported')
}

export function bindArtAtlasControls(bindings: Bindings): void {
  requireElement<HTMLSelectElement>('language').addEventListener('change', (event) => {
    bindings.applyLocale((event.currentTarget as HTMLSelectElement).value as ArtAtlasLocale)
  })
  requireElement<HTMLInputElement>('zoom').addEventListener('input', (event) => {
    bindings.updateSheetScale(Number((event.currentTarget as HTMLInputElement).value))
  })
  requireElement<HTMLInputElement>('guides').addEventListener('change', (event) => {
    bindings.sheetStage.classList.toggle('guides-hidden', !(event.currentTarget as HTMLInputElement).checked)
  })
  bindings.sourceCanvas.addEventListener('click', (event) => bindings.selectAt(event.clientX, event.clientY))
  requireElement<HTMLButtonElement>('reset').addEventListener('click', async () => {
    try {
      await bindings.resetPreview()
      bindings.setStatus('ready')
    } catch (error) { bindings.setStatus('ready', String(error)) }
  })
  requireElement<HTMLButtonElement>('load-files').addEventListener('click', () => void loadFiles(bindings))
  requireElement<HTMLButtonElement>('export').addEventListener('click', () => {
    void exportCurrent(bindings).catch((error: unknown) => bindings.setStatus('exported', String(error)))
  })
}
