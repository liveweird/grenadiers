import { create } from '@react-three/test-renderer'
import { useThree } from '@react-three/fiber'
import { useLayoutEffect } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { Camera, Material, Scene, WebGLRenderer } from 'three'
import { Mesh, WebGLRenderTarget } from 'three'
import { FinishPass } from './finishPass'

interface CapturedState { gl: WebGLRenderer; scene: Scene; camera: Camera }

function Capture({ stateRef }: { stateRef: { current: CapturedState | null } }) {
  const { gl, scene, camera } = useThree()
  useLayoutEffect(() => {
    stateRef.current = { gl, scene, camera }
  }, [camera, gl, scene, stateRef])
  return null
}

describe('FinishPass GPU lifetime', () => {
  it('resizes, renders in pass order, restores the default target, and disposes on unmount', async () => {
    const state: { current: CapturedState | null } = { current: null }
    const renderer = await create(<><FinishPass /><Capture stateRef={state} /></>)
    if (!state.current) throw new Error('Fiber state was not captured')
    const { gl, scene, camera } = state.current
    const drawingSize = { width: 640, height: 360 }
    vi.spyOn(gl, 'getDrawingBufferSize').mockImplementation((output) =>
      output.set(drawingSize.width, drawingSize.height))
    const setTarget = vi.spyOn(gl, 'setRenderTarget').mockImplementation(() => undefined)
    const clear = vi.spyOn(gl, 'clear').mockImplementation(() => undefined)
    const render = vi.spyOn(gl, 'render').mockImplementation(() => undefined)

    await renderer.advanceFrames(1, 1 / 60)
    const target = setTarget.mock.calls[0][0]
    expect(target).toBeInstanceOf(WebGLRenderTarget)
    expect(target?.width).toBe(640)
    expect(target?.height).toBe(360)
    expect(setTarget.mock.calls.map(([value]) => value)).toEqual([target, null])
    expect(clear).toHaveBeenCalledOnce()
    expect(render).toHaveBeenCalledTimes(2)
    expect(render.mock.calls[0]).toEqual([scene, camera])
    expect(render.mock.calls[1][0]).not.toBe(scene)

    const resize = vi.spyOn(target as WebGLRenderTarget, 'setSize')
    drawingSize.width = 800
    drawingSize.height = 600
    await renderer.advanceFrames(1, 1 / 60)
    expect(resize).toHaveBeenCalledWith(800, 600)

    const passMesh = (render.mock.calls[1][0] as Scene).children[0] as Mesh
    const targetDispose = vi.spyOn(target as WebGLRenderTarget, 'dispose')
    const passMaterial = (Array.isArray(passMesh.material) ? passMesh.material[0] : passMesh.material) as Material
    const materialDispose = vi.spyOn(passMaterial, 'dispose')
    const geometryDispose = vi.spyOn(passMesh.geometry, 'dispose')
    await renderer.unmount()
    expect(targetDispose).toHaveBeenCalledOnce()
    expect(materialDispose).toHaveBeenCalledOnce()
    expect(geometryDispose).toHaveBeenCalledOnce()
  })
})
