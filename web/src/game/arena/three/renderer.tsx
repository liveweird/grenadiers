import { createRoot, extend, unmountComponentAtNode } from '@react-three/fiber'
import { createElement } from 'react'
import {
  DirectionalLight, Group, HemisphereLight, InstancedMesh, Mesh, OrthographicCamera,
  Object3D,
} from 'three'
import type { RootStore } from '@react-three/fiber'
import type { ArenaRenderer } from '../rendererTypes'
import type { SnapshotMessage } from '../types'
import { createThreeAtlas } from './atlas'
import { createArenaCamera, fitArenaCamera } from './camera'
import { RendererRoot } from './RendererRoot'
import { createRoundCameraController } from './roundCamera'
import { createArenaSceneStore } from './sceneStore'

extend({ DirectionalLight, Group, HemisphereLight, InstancedMesh, Mesh, Object3D })

const ARENA_CLEAR_COLOR = 0x172127

function viewportSize(host: HTMLElement): { width: number; height: number } {
  const bounds = host.getBoundingClientRect()
  return {
    width: Math.max(1, Math.round(bounds.width || host.clientWidth || 720)),
    height: Math.max(1, Math.round(bounds.height || host.clientHeight || 624)),
  }
}

/** Creates an isolated manual Fiber root. The canvas stays hidden until a completed GPU frame. */
export async function createThreeArenaRenderer(
  host: HTMLElement, random: () => number = Math.random,
): Promise<ArenaRenderer> {
  const atlas = await createThreeAtlas()
  const canvas = document.createElement('canvas')
  canvas.setAttribute('aria-hidden', 'true')
  Object.assign(canvas.style, {
    width: '100%', height: '100%', display: 'block', visibility: 'hidden',
    imageRendering: 'pixelated',
  })
  let root: ReturnType<typeof createRoot> | null = null
  let rootStore: RootStore | null = null
  let resizeObserver: ResizeObserver | null = null
  let destroyed = false
  let wantedActive = false
  let contextLost = false
  let restoreRendererState = false
  let frameRequest = 0
  let asyncFault: unknown = null
  let board = { width: 15, height: 13 }
  const camera: OrthographicCamera = createArenaCamera()
  const store = createArenaSceneStore()
  const roundCamera = createRoundCameraController(random)

  const hideAndStop = (): void => {
    canvas.style.visibility = 'hidden'
    if (frameRequest !== 0) cancelAnimationFrame(frameRequest)
    frameRequest = 0
  }
  const frame = (timestamp: number): void => {
    frameRequest = 0
    if (destroyed || !wantedActive || contextLost || !rootStore) return
    try {
      const state = rootStore.getState()
      if (restoreRendererState) {
        // Our listener precedes Three's internal listener. Restore after its GL caches are rebuilt.
        state.gl.resetState()
        state.gl.setClearColor(ARENA_CLEAR_COLOR, 1)
        state.gl.shadowMap.needsUpdate = true
        atlas.texture.needsUpdate = true
        restoreRendererState = false
      }
      roundCamera.apply(camera, board, timestamp)
      state.advance(timestamp / 1000, true)
      canvas.style.visibility = 'visible'
      frameRequest = requestAnimationFrame(frame)
    } catch (error) {
      asyncFault ??= error
      wantedActive = false
      hideAndStop()
      store.clear()
      roundCamera.clear(camera, board)
    }
  }
  const scheduleFrame = (): void => {
    if (frameRequest === 0 && wantedActive && !contextLost && !destroyed) {
      frameRequest = requestAnimationFrame(frame)
    }
  }
  const resize = (): void => {
    if (!rootStore || destroyed) return
    const size = viewportSize(host)
    rootStore.getState().setSize(size.width, size.height, 0, 0)
    fitArenaCamera(camera, board, size)
    roundCamera.reapply(camera, board)
  }
  const onContextLost = (event: Event): void => {
    event.preventDefault()
    contextLost = true
    hideAndStop()
    roundCamera.clear(camera, board)
  }
  const onContextRestored = (): void => {
    if (destroyed) return
    contextLost = false
    restoreRendererState = true
    roundCamera.clear(camera, board)
    scheduleFrame()
  }
  const removeCanvas = (): void => {
    canvas.removeEventListener('webglcontextlost', onContextLost)
    canvas.removeEventListener('webglcontextrestored', onContextRestored)
    canvas.remove()
  }
  try {
    host.appendChild(canvas)
    canvas.addEventListener('webglcontextlost', onContextLost)
    canvas.addEventListener('webglcontextrestored', onContextRestored)
    root = createRoot(canvas)
    const size = viewportSize(host)
    fitArenaCamera(camera, board, size)
    await root.configure({
      camera,
      dpr: [1, Math.max(1, Math.min(window.devicePixelRatio || 1, 2))],
      flat: false,
      frameloop: 'never',
      gl: {
        alpha: false, antialias: true, depth: true, powerPreference: 'high-performance',
        stencil: false,
      },
      shadows: 'soft',
      size: { ...size, left: 0, top: 0 },
      onCreated: ({ gl }) => gl.setClearColor(ARENA_CLEAR_COLOR, 1),
    })
    let signalReady = (): void => undefined
    const committed = new Promise<void>((resolve) => { signalReady = resolve })
    rootStore = root.render(createElement(
      RendererRoot,
      { atlas, store, resolve: signalReady },
    ))
    await committed
    resizeObserver = typeof ResizeObserver === 'undefined' ? null : new ResizeObserver(resize)
    resizeObserver?.observe(host)
  } catch (error) {
    destroyed = true
    hideAndStop()
    resizeObserver?.disconnect()
    if (root) unmountComponentAtNode(canvas, () => atlas.dispose())
    else atlas.dispose()
    removeCanvas()
    throw error
  }

  const release = (): void => {
    hideAndStop()
    resizeObserver?.disconnect()
    resizeObserver = null
    store.clear()
    roundCamera.clear(camera, board)
    removeCanvas()
    if (root) unmountComponentAtNode(canvas, () => atlas.dispose())
    else atlas.dispose()
    root = null
    rootStore = null
  }
  return {
    update(snapshot: SnapshotMessage) {
      if (destroyed) return
      if (asyncFault) throw asyncFault
      const dimensionsChanged = board.width !== snapshot.state.arena.width
        || board.height !== snapshot.state.arena.height
      board = { width: snapshot.state.arena.width, height: snapshot.state.arena.height }
      if (dimensionsChanged) resize()
      const receivedAt = performance.now()
      roundCamera.update(snapshot, receivedAt)
      roundCamera.apply(camera, board, receivedAt)
      store.update(snapshot, receivedAt)
      wantedActive = true
      scheduleFrame()
    },
    suspend() {
      if (destroyed || !wantedActive) return
      wantedActive = false
      hideAndStop()
      store.clear()
      roundCamera.clear(camera, board)
    },
    setReducedMotion(reduced) {
      if (!destroyed) {
        roundCamera.setReducedMotion(reduced, camera, board)
        store.setReducedMotion(reduced)
      }
    },
    destroy() {
      if (destroyed) return
      destroyed = true
      wantedActive = false
      release()
    },
  }
}
