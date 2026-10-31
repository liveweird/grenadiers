import { useFrame } from '@react-three/fiber'
import { useEffect, useLayoutEffect, useMemo, useRef } from 'react'
import type { ReactNode } from 'react'
import { PointLight } from 'three'
import type { FireLightingModel } from './fireLighting'
import { FireLightingContext } from './fireLightingContext'

/** Four persistent slots avoid shader recompilation whenever flame counts change. */
export function FireIllumination({ model, animated, children }: {
  model: FireLightingModel
  animated: boolean
  children: ReactNode
}) {
  const lights = useMemo(() => Array.from({ length: 4 }, () => new PointLight(0xffa044, 0, 2.45)), [])
  const pool = useRef<readonly PointLight[]>([])
  useLayoutEffect(() => {
    pool.current = lights
    return () => { pool.current = [] }
  }, [lights])
  useEffect(() => () => { for (const light of lights) light.dispose() }, [lights])
  useFrame(() => {
    const now = performance.now()
    for (let index = 0; index < pool.current.length; index += 1) {
      const light = pool.current[index]
      const anchor = model.anchors[index]
      light.intensity = anchor
        ? anchor.intensity * (animated ? 1 + 0.07 * Math.sin(now / 170 + index * 2.1) : 1) : 0
      if (anchor) {
        light.position.set(...anchor.position)
        light.color.setHex(anchor.color)
        light.distance = anchor.distance
      }
    }
  })
  return <FireLightingContext value={model}>
    {lights.map((light, index) => <primitive key={index} object={light} dispose={null}
      name={`fire-light:${index}`} />)}
    {children}
  </FireLightingContext>
}
