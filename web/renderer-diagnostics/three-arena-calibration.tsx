/* eslint-disable react-refresh/only-export-components -- standalone diagnostic entry, never hot-reloaded */
import { createRoot, extend, unmountComponentAtNode, useFrame } from '@react-three/fiber'
import { useLayoutEffect } from 'react'
import {
  Mesh, MeshBasicMaterial, OrthographicCamera, PlaneGeometry,
} from 'three'
import { FinishPass } from '../src/game/arena/three/finishPass'

extend({ Mesh, MeshBasicMaterial, PlaneGeometry })

type ColorSample = { red: number; green: number; blue: number; alpha: number }
export type CalibrationSamples = {
  red: ColorSample
  green: ColorSample
  blue: ColorSample
  gray: ColorSample
}

const WIDTH = 128
const HEIGHT = 32
const sampleXs = { red: 16, green: 48, blue: 80, gray: 112 } as const

function CalibrationScene({ ready, capture }: {
  ready: () => void
  capture: (gl: WebGLRenderingContext | WebGL2RenderingContext) => void
}) {
  useLayoutEffect(ready, [ready])
  useFrame(({ gl }) => capture(gl.getContext()), 2)
  return <>
    <mesh position={[-1.5, 0, 0]}>
      <planeGeometry args={[1, 2]} />
      <meshBasicMaterial color={0xff2020} toneMapped={false} />
    </mesh>
    <mesh position={[-0.5, 0, 0]}>
      <planeGeometry args={[1, 2]} />
      <meshBasicMaterial color={0x20ff20} toneMapped={false} />
    </mesh>
    <mesh position={[0.5, 0, 0]}>
      <planeGeometry args={[1, 2]} />
      <meshBasicMaterial color={0x2020ff} toneMapped={false} />
    </mesh>
    <mesh position={[1.5, 0, 0]}>
      <planeGeometry args={[1, 2]} />
      <meshBasicMaterial color={0x808080} toneMapped={false} />
    </mesh>
    <FinishPass />
  </>
}

function readSample(gl: WebGLRenderingContext | WebGL2RenderingContext, x: number): ColorSample {
  const bytes = new Uint8Array(4)
  gl.readPixels(x, HEIGHT / 2, 1, 1, gl.RGBA, gl.UNSIGNED_BYTE, bytes)
  return { red: bytes[0], green: bytes[1], blue: bytes[2], alpha: bytes[3] }
}

export async function mountColorCalibration(host: HTMLElement): Promise<{
  samples: CalibrationSamples
  dispose: () => void
}> {
  const canvas = document.createElement('canvas')
  canvas.dataset.calibration = 'production-finish-pass'
  canvas.setAttribute('aria-label', 'Production GPU color calibration swatches')
  canvas.style.cssText = 'display:block;width:128px;height:32px;image-rendering:pixelated'
  host.replaceChildren(canvas)
  const root = createRoot(canvas)
  const camera = new OrthographicCamera(-2, 2, 1, -1, 0.1, 10)
  camera.position.set(0, 0, 2)
  camera.lookAt(0, 0, 0)
  camera.updateProjectionMatrix()
  camera.updateMatrixWorld()
  await root.configure({
    camera,
    dpr: 1,
    flat: false,
    frameloop: 'never',
    gl: { alpha: false, antialias: false, preserveDrawingBuffer: true },
    size: { width: WIDTH, height: HEIGHT, left: 0, top: 0 },
  })
  camera.left = -2
  camera.right = 2
  camera.top = 1
  camera.bottom = -1
  camera.updateProjectionMatrix()
  camera.updateMatrixWorld()
  let resolveReady = (): void => undefined
  const committed = new Promise<void>((resolve) => { resolveReady = resolve })
  let resolveSamples: ((samples: CalibrationSamples) => void) | undefined
  const sampled = new Promise<CalibrationSamples>((resolve) => { resolveSamples = resolve })
  let captured = false
  const capture = (gl: WebGLRenderingContext | WebGL2RenderingContext): void => {
    if (captured) return
    captured = true
    resolveSamples?.({
      red: readSample(gl, sampleXs.red),
      green: readSample(gl, sampleXs.green),
      blue: readSample(gl, sampleXs.blue),
      gray: readSample(gl, sampleXs.gray),
    })
  }
  const store = root.render(<CalibrationScene ready={resolveReady} capture={capture} />)
  await committed
  store.getState().advance(performance.now() / 1000, true)
  const samples = await sampled
  return {
    samples,
    dispose: () => unmountComponentAtNode(canvas, () => canvas.remove()),
  }
}
