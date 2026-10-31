import { describe, expect, it, vi } from 'vitest'

vi.mock('pixi.js', () => { throw new Error('pure atlas metadata loaded Pixi') })

describe('art atlas metadata dependency boundary', () => {
  it('builds and validates source and runtime manifests without loading Pixi', async () => {
    const [{ createSourceManifest }, { buildRuntimeManifest }, validators] = await Promise.all([
      import('./manifest'), import('./runtime'), import('./validate'),
    ])
    const source = createSourceManifest()
    const runtime = buildRuntimeManifest(source)

    expect(source.frames).toHaveLength(165)
    expect(validators.validateSourceManifest(source)).toEqual([])
    expect(validators.validateRuntimeManifest(runtime)).toEqual([])
  })
})
