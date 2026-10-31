import { create } from '@react-three/test-renderer'
import { describe, expect, it, vi } from 'vitest'
import { Texture } from 'three'
import type { ArenaSceneStore, ThreeAtlas } from './contracts'
import { RendererRoot } from './RendererRoot'

describe('RendererRoot commit signal', () => {
  it('signals readiness after mounting the scene subscription', async () => {
    const resolve = vi.fn()
    const revision = { snapshot: null, deaths: [], reducedMotion: false, receivedAt: 0, serial: 0 }
    const store: ArenaSceneStore = {
      read: () => revision,
      subscribe: () => () => undefined,
      update: vi.fn(), clear: vi.fn(), setReducedMotion: vi.fn(),
    }
    const atlas = {
      texture: new Texture(), dispose: vi.fn(),
    } as unknown as ThreeAtlas
    const renderer = await create(<RendererRoot atlas={atlas} store={store} resolve={resolve} />)
    expect(resolve).toHaveBeenCalledOnce()
    await renderer.unmount()
  })
})
