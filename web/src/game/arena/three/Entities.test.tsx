import ReactThreeTestRenderer from '@react-three/test-renderer'
import { BufferGeometry, Texture, Vector3, type Mesh, type MeshBasicMaterial } from 'three'
import { describe, expect, it, vi } from 'vitest'
import { gameSnapshot } from '../../../test/game-snapshot'
import { RENDERABLE_PICKUP_KINDS, type SnapshotMessage } from '../types'
import type { ArenaSceneStore, ThreeAtlas } from './contracts'
import { Entities } from './Entities'
import { createArenaSceneStore, readSceneTiming } from './sceneStore'

function testAtlas(): ThreeAtlas {
  const geometries = new Map<string, BufferGeometry>()
  return {
    texture: new Texture(),
    sourceManifest: {} as ThreeAtlas['sourceManifest'],
    manifest: {} as ThreeAtlas['manifest'],
    frame: (id) => ({
      id, sourceRect: { x: 0, y: 0, width: 80, height: 80 },
      localBounds: { x: 0, y: 0, width: 80, height: 80 }, pivot: { x: 40, y: 40 },
      ...(id === 'actor.shadow' ? { opacity: 0.3 } : {}),
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

function scene(
  snapshot: SnapshotMessage, store: ArenaSceneStore, atlas: ThreeAtlas,
) {
  const timing = readSceneTiming(store)
  if (!timing) throw new Error('test scene has no timing')
  return <Entities snapshot={snapshot} atlas={atlas} timing={timing}
    reducedMotion={store.read().reducedMotion} />
}

function actorNode(renderer: Awaited<ReturnType<typeof ReactThreeTestRenderer.create>>, id: number) {
  return renderer.scene.find(({ instance }) => instance.name === `actor:${id}`)
}

type Renderer = Awaited<ReturnType<typeof ReactThreeTestRenderer.create>>
const nodesNamed = (renderer: Renderer, name: string) =>
  renderer.scene.findAll(({ instance }) => instance.name === name)
const meshNamed = (renderer: Renderer, name: string) =>
  renderer.scene.find(({ instance }) => instance.name === name).instance as Mesh<BufferGeometry, MeshBasicMaterial>
const worldPosition = (instance: Renderer['scene']['instance']): Vector3 => {
  instance.updateWorldMatrix(true, false)
  return instance.getWorldPosition(new Vector3())
}

describe('mounted Three entities', () => {
  it('remounts after coalesced hidden and scene-reset revisions instead of crossing concealed space', async () => {
    const atlas = testAtlas()
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.bombs = []; snapshot.state.flames = []; snapshot.state.pickups = []
    snapshot.state.oilBarrels = []; snapshot.state.mobs = []; snapshot.state.lairs = []
    const actor = { ...snapshot.state.actors[0], position: { x: 1536, y: 1536 } }
    snapshot.state.actors = [actor]
    store.update(snapshot, 0)
    const renderer = await ReactThreeTestRenderer.create(scene(snapshot, store, atlas))
    const first = actorNode(renderer, actor.id).instance
    expect(first.position.x).toBeCloseTo(1.5)

    snapshot.state.actors = []
    store.update(snapshot, 20)
    snapshot.state.actors = [{ ...actor, position: { x: 5632, y: 1536 } }]
    store.update(snapshot, 40)
    await renderer.update(scene(snapshot, store, atlas))
    const revealed = actorNode(renderer, actor.id).instance
    expect(revealed).not.toBe(first)
    expect(revealed.position.x).toBeCloseTo(5.5)

    snapshot.room.roundId += 1
    snapshot.state.actors[0].position.x = 6656
    store.update(snapshot, 60)
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 1
    snapshot.state.actors[0].position.x = 7680
    store.update(snapshot, 80)
    await renderer.update(scene(snapshot, store, atlas))
    const reset = actorNode(renderer, actor.id).instance
    expect(reset).not.toBe(revealed)
    expect(reset.position.x).toBeCloseTo(7.5)

    store.clear()
    snapshot.state.actors[0].position.x = 8704
    store.update(snapshot, 100)
    await renderer.update(scene(snapshot, store, atlas))
    const cleared = actorNode(renderer, actor.id).instance
    expect(cleared).not.toBe(reset)
    expect(cleared.position.x).toBeCloseTo(8.5)
    await renderer.unmount()
    atlas.dispose()
  })

  it('applies the snapshot-derived timed bomb shell pulse scale', async () => {
    const atlas = testAtlas()
    const store = createArenaSceneStore()
    const snapshot = gameSnapshot()
    snapshot.state.actors = []; snapshot.state.flames = []; snapshot.state.pickups = []
    snapshot.state.oilBarrels = []; snapshot.state.mobs = []; snapshot.state.lairs = []
    snapshot.state.bombs = [{
      id: 7, cell: { column: 2, row: 3 }, owner: 1, range: 2, placedAt: 0,
      trigger: { type: 'timed', remainingTicks: 48, paused: false }, blast: 'standard',
    }]
    store.update(snapshot, 0)
    const renderer = await ReactThreeTestRenderer.create(scene(snapshot, store, atlas))
    const shell = renderer.scene.find(({ instance }) =>
      instance.name === 'billboard:bomb.shell.timed.standard.2').instance
    expect(shell.scale.x).toBeCloseTo(1.125 / 48)
    await renderer.unmount()
    atlas.dispose()
  })
})

describe('mounted Three entity animation and variants', () => {
  it('mounts the complete public entity showcase with trigger-specific and deduplicated layers', async () => {
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const atlas = testAtlas(); const store = createArenaSceneStore(); const snapshot = gameSnapshot()
    const actor = snapshot.state.actors[0]
    Object.assign(actor, {
      damageGraceRemainingTicks: 1, shieldRemainingTicks: 1, stopwatchRemainingTicks: 1,
      voluntaryActionsFrozen: true, invisibilityCloakRemainingTicks: 1,
      goldenFlameRemainingTicks: 1, shortFuseRemainingTicks: 1, shortFlameRemainingTicks: 1,
      longFuseRemainingTicks: 1, reverseControlsRemainingTicks: 1, diarrheaRemainingTicks: 1,
      slowFootRemainingTicks: [1, 1], stormSlowRemainingTicks: 1,
      invisibleBombsRemainingTicks: 1, piercingArmed: true, powerArmed: true,
      landmineStored: true, remoteArmed: true, kickEquipped: true, punchEquipped: true,
      braceletEquipped: true, wallPassEquipped: true,
    })
    snapshot.state.actors = [actor]
    snapshot.state.pickups = RENDERABLE_PICKUP_KINDS.map((kind, index) => ({
      kind, cell: { column: index % 13 + 1, row: Math.floor(index / 13) + 1 },
    }))
    snapshot.state.bombs = [
      { id: 1, cell: { column: 1, row: 3 }, owner: 1, range: 1, placedAt: 0,
        trigger: { type: 'timed', remainingTicks: 48, paused: false }, blast: 'piercing' },
      { id: 2, cell: { column: 2, row: 3 }, owner: 1, range: 1, placedAt: 0,
        trigger: { type: 'timed', remainingTicks: 48, paused: true }, blast: 'power' },
      { id: 3, cell: { column: 3, row: 3 }, owner: 1, range: 1, placedAt: 0,
        trigger: { type: 'remote' }, blast: 'power' },
      { id: 4, cell: { column: 4, row: 3 }, owner: 1, range: 1, placedAt: 0,
        trigger: { type: 'contact' }, blast: 'standard' },
    ]
    snapshot.state.flames = [
      { cell: { column: 5, row: 5 }, expiresAt: 200 },
      { cell: { column: 6, row: 5 }, expiresAt: 200 },
    ]
    snapshot.state.oilBarrels = [
      { cell: { column: 6, row: 5 }, state: 'burning' },
      { cell: { column: 7, row: 5 }, state: 'unlit' },
    ]
    snapshot.state.mobs = [{
      id: 9, lairId: 1, position: { x: 4096, y: 4096 }, heading: 'north', frozen: false,
    }]
    snapshot.state.stormCloud = {
      position: { x: 5120, y: 4096 }, direction: 'east', legRemainingTicks: 10,
    }
    store.update(snapshot, 0)
    const renderer = await ReactThreeTestRenderer.create(scene(snapshot, store, atlas))

    for (const kind of RENDERABLE_PICKUP_KINDS) {
      expect(nodesNamed(renderer, `billboard:pickup.${kind}`))
        .toHaveLength(kind === 'invisibleBombs' ? 2 : 1)
    }
    expect(nodesNamed(renderer, 'billboard:bomb.shell.timed.piercing.2')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:bomb.spark')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:bomb.pause')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:bomb.shell.remote.power')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:bomb.shell.contact.standard')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:oil.charred')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:oil.unlit')).toHaveLength(1)
    expect(nodesNamed(renderer, 'billboard:effect.flame.0')).toHaveLength(2)
    expect(nodesNamed(renderer, 'mob:9')).toHaveLength(1)
    expect(nodesNamed(renderer, 'storm-cloud')).toHaveLength(1)
    for (const cue of ['protection', 'shield', 'stopwatch', 'frozen', 'goldenFlame',
      'shortFuse', 'shortFlame', 'longFuse', 'reverseControls', 'diarrhea', 'slowBoot',
      'piercing', 'power', 'landmine', 'kick', 'punch', 'bracelet', 'wallPass', 'cloak']) {
      expect(nodesNamed(renderer, `billboard:actor.cue.${cue}`)).toHaveLength(1)
    }
    await renderer.advanceFrames(1, 0.016)
    expect(meshNamed(renderer, 'billboard:actor.cue.protection').material.opacity).toBeCloseTo(0.65)
    expect((nodesNamed(renderer, 'billboard:actor.shadow')[0].instance as Mesh<BufferGeometry, MeshBasicMaterial>)
      .material.opacity).toBeCloseTo(0.3)
    now = 120
    await renderer.advanceFrames(1, 0.016)
    expect(meshNamed(renderer, 'billboard:actor.cue.protection').material.opacity).toBeCloseTo(1)
    for (const flame of nodesNamed(renderer, 'billboard:effect.flame.0')) {
      expect((flame.instance as Mesh).geometry).toBe(atlas.geometry('effect.flame.1', 'center'))
    }
    store.setReducedMotion(true)
    delete snapshot.state.oilBarrels
    await renderer.update(scene(snapshot, store, atlas))
    now = 200
    await renderer.advanceFrames(1, 0.016)
    expect(nodesNamed(renderer, 'billboard:oil.charred')).toHaveLength(0)
    for (const flame of nodesNamed(renderer, 'billboard:effect.flame.0')) {
      expect((flame.instance as Mesh).geometry).toBe(atlas.geometry('effect.flame.0', 'center'))
    }
    await renderer.unmount(); atlas.dispose(); vi.restoreAllMocks()
  })

  it('animates received movement, then snaps frozen and reduced-motion entities', async () => {
    let now = 0
    vi.spyOn(performance, 'now').mockImplementation(() => now)
    const atlas = testAtlas(); const store = createArenaSceneStore(); const snapshot = gameSnapshot()
    snapshot.state.actors = [snapshot.state.actors[0]]; snapshot.state.bombs = []
    snapshot.state.flames = []; snapshot.state.pickups = []; snapshot.state.oilBarrels = []
    snapshot.state.mobs = [{
      id: 9, lairId: 1, position: { x: 4096, y: 4096 }, heading: 'east', frozen: false,
    }]
    snapshot.state.stormCloud = {
      position: { x: 5120, y: 4096 }, direction: 'east', legRemainingTicks: 10,
    }
    store.update(snapshot, 0)
    const renderer = await ReactThreeTestRenderer.create(scene(snapshot, store, atlas))
    const actorId = snapshot.state.actors[0].id
    const restingActor = actorNode(renderer, actorId).instance
    snapshot.state.physicalTick = (snapshot.state.physicalTick ?? 0) + 3
    snapshot.state.actors[0].position.x += 1024
    snapshot.state.mobs[0].position.x += 1024
    snapshot.state.stormCloud!.position.x += 1024
    store.update(snapshot, 100)
    await renderer.update(scene(snapshot, store, atlas))
    expect(actorNode(renderer, actorId).instance).toBe(restingActor)
    expect(actorNode(renderer, actorId).instance.position.x).toBeCloseTo(1.5)
    now = 125
    await renderer.advanceFrames(1, 0.025)
    expect(actorNode(renderer, actorId).instance.position.x).toBeCloseTo(2)
    expect(meshNamed(renderer, 'billboard:actor.one.east.0').geometry)
      .toBe(atlas.geometry('actor.one.east.3', 'feet'))
    expect(renderer.scene.find(({ instance }) => instance.name === 'mob:9').instance.position.x)
      .toBeCloseTo(4.5)
    expect(renderer.scene.find(({ instance }) => instance.name === 'storm-cloud').instance.position.x)
      .toBeCloseTo(5.5)

    const movingMob = renderer.scene.find(({ instance }) => instance.name === 'mob:9').instance
    snapshot.state.physicalTick += 3
    snapshot.state.mobs[0].position.x += 1024
    snapshot.state.mobs[0].frozen = true
    store.update(snapshot, 200)
    await renderer.update(scene(snapshot, store, atlas))
    const frozenMob = renderer.scene.find(({ instance }) => instance.name === 'mob:9').instance
    expect(frozenMob).not.toBe(movingMob)
    expect(frozenMob.position.x).toBeCloseTo(6)

    const floatingCloud = renderer.scene.find(({ instance }) => instance.name === 'storm-cloud')
    const cloudSprite = floatingCloud.find(({ instance }) =>
      instance.name === 'billboard:environment.stormCloud').instance
    const cloudShadow = floatingCloud.find(({ instance }) =>
      instance.name === 'billboard:actor.shadow').instance
    now = Math.PI * 260 / 2
    await renderer.advanceFrames(1, 0.016)
    expect(worldPosition(cloudSprite).y).toBeCloseTo(1.23)
    expect(worldPosition(cloudShadow).y).toBeCloseTo(0.003)
    now = Math.PI * 260 * 3 / 2
    await renderer.advanceFrames(1, 0.016)
    expect(worldPosition(cloudSprite).y).toBeCloseTo(1.07)
    expect(worldPosition(cloudShadow).y).toBeCloseTo(0.003)
    store.setReducedMotion(true)
    snapshot.state.physicalTick += 3
    snapshot.state.actors[0].position.x += 1024
    store.update(snapshot, 300)
    await renderer.update(scene(snapshot, store, atlas))
    await renderer.advanceFrames(1, 0.016)
    expect(actorNode(renderer, actorId).instance.position.x).toBeCloseTo(3.5)
    expect(meshNamed(renderer, 'billboard:actor.one.east.0').geometry)
      .toBe(atlas.geometry('actor.one.east.0', 'feet'))
    const reducedCloud = renderer.scene.find(({ instance }) => instance.name === 'storm-cloud')
    expect(worldPosition(reducedCloud.find(({ instance }) =>
      instance.name === 'billboard:environment.stormCloud').instance).y).toBeCloseTo(1.15)
    expect(worldPosition(reducedCloud.find(({ instance }) =>
      instance.name === 'billboard:actor.shadow').instance).y).toBeCloseTo(0.003)
    await renderer.unmount(); atlas.dispose(); vi.restoreAllMocks()
  })
})
