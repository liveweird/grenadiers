import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../test/game-snapshot'
import { ArenaScene } from './scene'

vi.mock('pixi.js', async () => (await import('../../test/arena-pixi-mock')).arenaPixiMock())

type NodeView = {
  children: NodeView[]
  x: number
  y: number
  alpha: number
  visible: boolean
  destroyed: boolean
}

function actorLayer(scene: ArenaScene): NodeView {
  return scene.root.children[5] as unknown as NodeView
}

describe('Invisibility Cloak retained-scene privacy', () => {
  it('destroys every opponent drawing while hidden and reveals at the current position', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors[0].invisibilityCloakRemainingTicks = 600
    scene.update(snapshot, 0)
    const layer = actorLayer(scene)
    const cloakDrawing = layer.children[0].children.find(
      (child) => child.visible && child.alpha > 0.69 && child.alpha < 0.71,
    )
    expect(cloakDrawing).toBeDefined()
    scene.setReducedMotion(true)
    scene.render(1)
    expect(cloakDrawing).toMatchObject({ visible: true, alpha: 1 })
    const hiddenRoot = layer.children[1]
    const hiddenChildren = [...hiddenRoot.children]

    snapshot.state.physicalTick = 101
    snapshot.state.actors = [snapshot.state.actors[0]]
    scene.update(snapshot, 20)
    expect(layer.children).toHaveLength(1)
    expect(hiddenRoot.destroyed).toBe(true)
    expect(hiddenChildren.every((child) => child.destroyed)).toBe(true)

    snapshot.state.physicalTick = 102
    snapshot.state.actors.push({
      id: 20, seat: 'two', position: { x: 4096, y: 3072 }, heading: 'north', alive: true,
    })
    scene.update(snapshot, 40)
    expect(layer.children).toHaveLength(2)
    expect(layer.children[1]).toMatchObject({ x: 192, y: 144 })
    scene.render(41)
    expect(layer.children[1]).toMatchObject({ x: 192, y: 144 })
  })

  it('draws a distinct cloak pickup without adding an opponent actor root', () => {
    const scene = new ArenaScene(1)
    const snapshot = gameSnapshot()
    snapshot.state.actors = [snapshot.state.actors[0]]
    snapshot.state.pickups = [{ cell: { column: 3, row: 1 }, kind: 'invisibilityCloak' }]
    scene.update(snapshot, 0)
    const pickups = scene.root.children[2] as unknown as NodeView
    expect(pickups.children).toHaveLength(1)
    expect(actorLayer(scene).children).toHaveLength(1)
  })
})
