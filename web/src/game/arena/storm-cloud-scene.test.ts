import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = { children: NodeView[]; x: number; y: number; destroyed: boolean }
const layers = (scene: ArenaScene): NodeView[] => scene.root.children as unknown as NodeView[]

describe('storm cloud arena entity', () => {
  it('interpolates only between received cloud positions and removes the dedicated layer on null', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.stormCloud = {
      position: { x: 7_680, y: 6_656 }, direction: 'east', legRemainingTicks: 128,
    }
    scene.update(snapshot, 0)
    expect(layers(scene)).toHaveLength(7)
    const weather = layers(scene)[5]
    const cloud = weather.children[0]
    expect(cloud).toMatchObject({ x: 360, y: 312 })
    expect(layers(scene)[6].children).toHaveLength(2)

    snapshot.state.physicalTick = 103
    snapshot.state.stormCloud.position.x += 1_024
    snapshot.state.stormCloud.legRemainingTicks = 0
    scene.update(snapshot, 100)
    expect(cloud.x).toBe(360)
    scene.render(125)
    expect(cloud.x).toBe(384)
    scene.render(100_000)
    expect(cloud.x).toBe(408)

    snapshot.state.stormCloud = null
    scene.update(snapshot, 200)
    expect(weather.destroyed).toBe(true)
    expect(layers(scene)).toHaveLength(6)
  })

  it('snaps reduced motion and world resets without retaining resources', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.stormCloud = {
      position: { x: 7_680, y: 6_656 }, direction: 'north', legRemainingTicks: 64,
    }
    scene.update(snapshot, 0)
    const firstWeather = layers(scene)[5]
    const firstCloud = firstWeather.children[0]

    scene.setReducedMotion(true)
    snapshot.state.physicalTick = 103
    snapshot.state.stormCloud.position.y -= 512
    scene.update(snapshot, 100)
    expect(firstCloud.y).toBe(288)

    scene.setReducedMotion(false)
    snapshot.room.roundId += 1
    snapshot.state.stormCloud.position = { x: 6_656, y: 5_632 }
    scene.update(snapshot, 200)
    expect(firstWeather.destroyed).toBe(true)
    expect(layers(scene)[5].children[0]).toMatchObject({ x: 312, y: 264 })

    const replacement = layers(scene)[5]
    scene.reset()
    expect(replacement.destroyed).toBe(true)
    expect(layers(scene)).toHaveLength(6)
  })
})
