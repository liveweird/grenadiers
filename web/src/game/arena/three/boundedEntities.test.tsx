import ReactThreeTestRenderer from '@react-three/test-renderer'
import { BufferGeometry, Texture, type Mesh } from 'three'
import { describe, expect, it } from 'vitest'
import { initialGameClientState } from '../../client/clientTypes'
import { presentSnapshot } from '../../client/presentation'
import { snapshot } from '../../client/testSupport'
import { ActorVisual } from './ActorVisual'
import { actorFrameId, createActorView } from './actorView'
import type { ThreeAtlas } from './contracts'
import { motionTiming } from './motion'
import { gameSnapshot } from '../../../test/game-snapshot'

function testAtlas(): ThreeAtlas {
  const geometries = new Map<string, BufferGeometry>()
  return {
    texture: new Texture(),
    sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({
      id,
      sourceRect: { x: 0, y: 0, width: id.startsWith('pickup.') ? 48 : 80, height: 80 },
      localBounds: { x: 0, y: 0, width: 80, height: 80 },
      pivot: { x: 40, y: 40 },
    }),
    geometry(id, anchor = 'pivot') {
      const key = `${id}:${anchor}`
      const geometry = geometries.get(key) ?? new BufferGeometry()
      geometries.set(key, geometry)
      return geometry
    },
    dispose() {},
  }
}

const timing = motionTiming(gameSnapshot(), 0, null)

function presentedRemoteOwner(invisibleBombs: boolean) {
  const wire = snapshot(2)
  const own = wire.state.actors[0]
  if (own.type !== 'visible') throw new Error('expected visible own actor')
  own.inventory.weapon = 'powerBomb'
  own.effects = invisibleBombs ? [{ kind: 'invisibleBombs', remainingTicks: 120 }] : []
  wire.state.bombs = [{
    id: 3, cell: { column: 3, row: 1 }, owner: own.id, range: 2, placedAt: 1,
    trigger: { type: 'remote' }, blast: 'standard',
  }]
  const presented = presentSnapshot({
    ...initialGameClientState(), connection: 'authenticated', room: wire.room, snapshot: wire,
    youSeat: 'one', admissionBaseline: {
      roomId: wire.room.roomId, revision: 1, tick: 1, physicalTick: 1,
    },
  })
  if (!presented) throw new Error('expected presented snapshot')
  return presented.state.actors[0]
}

describe('bounded actor status badges', () => {
  it('keeps the uniform visible and mounts a small remote-bomb badge beside the head', async () => {
    const atlas = testAtlas()
    const actor = presentedRemoteOwner(false)
    actor.damageGraceRemainingTicks = 1
    const view = createActorView(actor, false)
    const renderer = await ReactThreeTestRenderer.create(
      <ActorVisual atlas={atlas} view={view} timing={timing} reducedMotion />,
    )
    const bodyId = actorFrameId(view, 0, false)
    const body = renderer.scene.find(({ instance }) =>
      instance.name === `billboard:${bodyId}`).instance as Mesh
    const badge = renderer.scene.find(({ instance }) =>
      instance.name === 'billboard:bomb.shell.remote.standard').instance as Mesh
    const protection = renderer.scene.find(({ instance }) =>
      instance.name === 'billboard:actor.cue.protection').instance as Mesh

    expect(body.geometry).toBe(atlas.geometry(bodyId, 'feet'))
    expect(protection.scale.x).toBe(body.scale.x)
    expect(protection.position.x).toBe(0)
    expect(badge.position.x).toBeGreaterThan(0)
    expect(badge.position.y).toBeGreaterThan(1)
    expect(badge.renderOrder).toBeGreaterThan(body.renderOrder)
    const bodyWidth = atlas.frame(bodyId).sourceRect.width * body.scale.x
    const badgeWidth = atlas.frame('bomb.shell.remote.standard').sourceRect.width * badge.scale.x
    expect(badgeWidth).toBeLessThan(bodyWidth * 0.35)

    await renderer.unmount()
  })

  it('shows only the compact Invisible Bombs badge when bomb ownership is private', async () => {
    const atlas = testAtlas()
    const actor = presentedRemoteOwner(true)
    const view = createActorView(actor, false)
    const renderer = await ReactThreeTestRenderer.create(
      <ActorVisual atlas={atlas} view={view} timing={timing} reducedMotion />,
    )

    expect(renderer.scene.findAll(({ instance }) =>
      instance.name === 'billboard:bomb.shell.remote.standard')).toHaveLength(0)
    const badge = renderer.scene.find(({ instance }) =>
      instance.name === 'billboard:pickup.invisibleBombs').instance as Mesh
    const bodyId = actorFrameId(view, 0, false)
    const body = renderer.scene.find(({ instance }) =>
      instance.name === `billboard:${bodyId}`).instance as Mesh
    const bodyWidth = atlas.frame(bodyId).sourceRect.width * body.scale.x
    const badgeWidth = atlas.frame('pickup.invisibleBombs').sourceRect.width * badge.scale.x
    expect(badgeWidth).toBeLessThan(bodyWidth * 0.25)

    await renderer.unmount()
  })

  it('separates simultaneous badges on a visible opponent', async () => {
    const atlas = testAtlas()
    // A recipient without Invisible Bombs can see both cues on an affected opponent.
    const actor = { ...presentedRemoteOwner(false), invisibleBombsRemainingTicks: 120 }
    const renderer = await ReactThreeTestRenderer.create(
      <ActorVisual atlas={atlas} view={createActorView(actor, false)} timing={timing} reducedMotion />,
    )
    const badge = (id: string) => renderer.scene.find(({ instance }) =>
      instance.name === `billboard:${id}`).instance as Mesh
    const remote = badge('bomb.shell.remote.standard')
    const invisible = badge('pickup.invisibleBombs')
    const halfWidth = (id: string, mesh: Mesh) => atlas.frame(id).sourceRect.width * mesh.scale.x / 2
    expect(invisible.position.x + halfWidth('pickup.invisibleBombs', invisible))
      .toBeLessThan(remote.position.x - halfWidth('bomb.shell.remote.standard', remote))
    await renderer.unmount()
  })
})
