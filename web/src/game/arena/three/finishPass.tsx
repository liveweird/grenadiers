import { useEffect, useMemo } from 'react'
import { useFrame } from '@react-three/fiber'
import {
  Mesh, OrthographicCamera, PlaneGeometry, Scene, ShaderMaterial, Vector2, WebGLRenderTarget,
} from 'three'
import { finishFragmentShader, finishVertexShader } from './finishShader'

/** Restrained static grade and sub-LSB ordered dither; owns no temporal history. */
export function FinishPass() {
  const pass = useMemo(() => {
    const target = new WebGLRenderTarget(1, 1, { depthBuffer: true, stencilBuffer: false })
    target.texture.name = 'arena-finish-source'
    const material = new ShaderMaterial({
      uniforms: { sourceMap: { value: target.texture } },
      vertexShader: finishVertexShader,
      fragmentShader: finishFragmentShader,
      depthTest: false, depthWrite: false, toneMapped: false,
    })
    const geometry = new PlaneGeometry(2, 2)
    const scene = new Scene()
    scene.add(new Mesh(geometry, material))
    const camera = new OrthographicCamera(-1, 1, 1, -1, 0, 1)
    return { target, material, geometry, scene, camera, size: new Vector2() }
  }, [])
  useEffect(() => () => {
    pass.target.dispose()
    pass.material.dispose()
    pass.geometry.dispose()
  }, [pass])
  useFrame(({ gl, scene, camera }) => {
    gl.getDrawingBufferSize(pass.size)
    const width = Math.max(1, pass.size.x)
    const height = Math.max(1, pass.size.y)
    if (pass.target.width !== width || pass.target.height !== height) pass.target.setSize(width, height)
    gl.setRenderTarget(pass.target)
    gl.clear()
    gl.render(scene, camera)
    gl.setRenderTarget(null)
    gl.render(pass.scene, pass.camera)
  }, 1)
  return null
}
