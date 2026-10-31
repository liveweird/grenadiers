import { useCallback, useLayoutEffect, useMemo, useRef, type ReactNode } from 'react'
import {
  DirectionalLight, Euler, InstancedMesh, Matrix4, Object3D, Quaternion, Vector3,
} from 'three'
import type { SnapshotMessage } from '../types'
import type { ThreeAtlas } from './contracts'
import { cellToWorld, WORLD_TILE } from './coordinates'
import { aimArenaLight, arenaLightPlacement } from './lighting'
import { createWorldResources, replaceInstancedMesh } from './resources'

const FLOOR_Y = 0
const WALL_HEIGHT = 0.48
const BLOCK_HEIGHT = 0.54
const LAIR_HEIGHT = 0.8
const RIM_HEIGHT = 0.22
const WALL_BEVEL = 0.055
const BLOCK_BEVEL = 0.06
const LAIR_BEVEL = 0.045
const RIM_BEVEL = 0.04
const ART_TILE = 48

interface Instance {
  position: [number, number, number]
  rotation: [number, number, number]
  scale: [number, number, number]
}

interface InstancesProps {
  geometry: ReturnType<ThreeAtlas['geometry']>
  material: ReturnType<typeof createWorldResources>['material']
  instances: readonly Instance[]
  castShadow?: boolean
}

function Instances({ geometry, material, instances, castShadow = false }: InstancesProps) {
  const mesh = useRef<InstancedMesh>(null)
  const setMesh = useCallback((next: InstancedMesh | null) => {
    mesh.current = replaceInstancedMesh(mesh.current, next)
  }, [])
  useLayoutEffect(() => {
    if (!mesh.current) return
    const matrix = new Matrix4()
    const quaternion = new Quaternion()
    const position = new Vector3()
    const scale = new Vector3()
    const rotation = new Euler()
    instances.forEach((instance, index) => {
      position.set(...instance.position)
      rotation.set(...instance.rotation)
      quaternion.setFromEuler(rotation)
      scale.set(...instance.scale)
      matrix.compose(position, quaternion, scale)
      mesh.current?.setMatrixAt(index, matrix)
    })
    mesh.current.instanceMatrix.needsUpdate = true
    mesh.current.computeBoundingSphere()
  }, [instances])
  if (instances.length === 0) return null
  return <instancedMesh
    ref={setMesh} args={[geometry, material, instances.length]}
    dispose={null}
    castShadow={castShadow} receiveShadow frustumCulled
  />
}

function topInstance(x: number, z: number, y: number, scale = 1): Instance {
  return {
    position: [x, y, z], rotation: [-Math.PI / 2, 0, 0],
    scale: [WORLD_TILE * scale / ART_TILE, WORLD_TILE * scale / ART_TILE, 1],
  }
}

function boxInstances(
  x: number, z: number, height: number,
  shape: { width: number; bevel: number; base?: number },
): Instance[] {
  const { width, bevel, base = 0 } = shape
  const half = width / 2
  const sideHeight = height - base - bevel
  const horizontal: [number, number, number] = [width / ART_TILE, sideHeight / ART_TILE, 1]
  const centerY = base + sideHeight / 2
  return [
    { position: [x, centerY, z + half], rotation: [0, 0, 0], scale: horizontal },
    { position: [x, centerY, z - half], rotation: [0, Math.PI, 0], scale: horizontal },
    { position: [x + half, centerY, z], rotation: [0, Math.PI / 2, 0], scale: horizontal },
    { position: [x - half, centerY, z], rotation: [0, -Math.PI / 2, 0], scale: horizontal },
  ]
}

function chamferInstances(x: number, z: number, height: number, width: number, bevel: number): Instance[] {
  const half = width / 2
  const centerY = height - bevel / 2
  const inset = half - bevel / 2
  const scale: [number, number, number] = [width / ART_TILE, Math.SQRT2 * bevel / ART_TILE, 1]
  return [
    { position: [x, centerY, z + inset], rotation: [-Math.PI / 4, 0, 0], scale },
    { position: [x, centerY, z - inset], rotation: [-Math.PI / 4, Math.PI, 0], scale },
    { position: [x + inset, centerY, z], rotation: [-Math.PI / 4, Math.PI / 2, 0], scale },
    { position: [x - inset, centerY, z], rotation: [-Math.PI / 4, -Math.PI / 2, 0], scale },
  ]
}

function beveledSides(
  x: number, z: number, height: number,
  shape: { width: number; bevel: number; base?: number },
): Instance[] {
  const { width, bevel } = shape
  return [
    ...boxInstances(x, z, height, shape),
    ...chamferInstances(x, z, height, width, bevel),
  ]
}

interface WorldModel {
  floors: [Instance[], Instance[]]
  walls: { tops: Instance[]; sides: Instance[] }
  blocks: { tops: Instance[]; sides: Instance[] }
  lairs: { tops: Instance[]; sides: Instance[]; entrances: Instance[] }
  rim: { tops: Instance[]; sides: Instance[] }
}

function createWorldModel(snapshot: SnapshotMessage): WorldModel {
  const { arena } = snapshot.state
  const { blocks } = arena
  const floors: [Instance[], Instance[]] = [[], []]
  const walls = { tops: [] as Instance[], sides: [] as Instance[] }
  const lairCells = new Set(snapshot.state.lairs.map(
    ({ cell }) => cell.column + cell.row * arena.width,
  ))
  arena.terrain.forEach((tile, index) => {
    if (lairCells.has(index)) return
    const column = index % arena.width
    const row = Math.floor(index / arena.width)
    const [x, z] = cellToWorld(column, row)
    if (tile === 'floor') floors[(column * 3 + row * 5) & 1].push(topInstance(x, z, FLOOR_Y))
    else {
      walls.tops.push(topInstance(x, z, WALL_HEIGHT, 1 - WALL_BEVEL * 2))
      walls.sides.push(...beveledSides(x, z, WALL_HEIGHT, { width: WORLD_TILE, bevel: WALL_BEVEL }))
    }
  })
  const blockModel = { tops: [] as Instance[], sides: [] as Instance[] }
  for (const block of blocks) {
    const [x, z] = cellToWorld(block.column, block.row)
    const width = WORLD_TILE * 0.84
    blockModel.tops.push(topInstance(x, z, BLOCK_HEIGHT, width - BLOCK_BEVEL * 2))
    blockModel.sides.push(...beveledSides(x, z, BLOCK_HEIGHT, { width, bevel: BLOCK_BEVEL }))
  }
  const lairs = { tops: [] as Instance[], sides: [] as Instance[], entrances: [] as Instance[] }
  for (const lair of snapshot.state.lairs) {
    const [x, z] = cellToWorld(lair.cell.column, lair.cell.row)
    const width = WORLD_TILE * 0.82
    lairs.tops.push(topInstance(x, z, LAIR_HEIGHT, width - LAIR_BEVEL * 2))
    lairs.sides.push(...beveledSides(x, z, LAIR_HEIGHT, { width, bevel: LAIR_BEVEL }))
    const entranceSize = WORLD_TILE * 0.7
    lairs.entrances.push({
      position: [x, (LAIR_HEIGHT - LAIR_BEVEL) / 2, z + width / 2 + 0.003],
      rotation: [0, 0, 0],
      scale: [entranceSize / ART_TILE, entranceSize / ART_TILE, 1],
    })
  }
  const rim = { tops: [] as Instance[], sides: [] as Instance[] }
  for (let column = -1; column <= arena.width; column += 1) {
    for (const row of [-1, arena.height]) {
      const [x, z] = cellToWorld(column, row)
      rim.tops.push(topInstance(x, z, RIM_HEIGHT, 1 - RIM_BEVEL * 2))
      rim.sides.push(...beveledSides(x, z, RIM_HEIGHT, {
        width: WORLD_TILE, bevel: RIM_BEVEL, base: -0.12,
      }))
    }
  }
  for (let row = 0; row < arena.height; row += 1) {
    for (const column of [-1, arena.width]) {
      const [x, z] = cellToWorld(column, row)
      rim.tops.push(topInstance(x, z, RIM_HEIGHT, 1 - RIM_BEVEL * 2))
      rim.sides.push(...beveledSides(x, z, RIM_HEIGHT, {
        width: WORLD_TILE, bevel: RIM_BEVEL, base: -0.12,
      }))
    }
  }
  return { floors, walls, blocks: blockModel, lairs, rim }
}

export interface ArenaWorldProps {
  snapshot: SnapshotMessage
  atlas: ThreeAtlas
  children?: ReactNode
}

export function ArenaWorld({ snapshot, atlas, children }: ArenaWorldProps) {
  const resources = useMemo(() => createWorldResources(atlas), [atlas])
  useLayoutEffect(() => () => resources.dispose(), [resources])
  // Published snapshots may reuse their outer container, so do not key instances on identity.
  const model = createWorldModel(snapshot)
  const material = resources.material
  const light = useRef<DirectionalLight>(null)
  const lightTarget = useRef<Object3D>(null)
  const { width, height } = snapshot.state.arena
  const lighting = arenaLightPlacement(width, height)
  useLayoutEffect(() => {
    if (light.current && lightTarget.current) aimArenaLight(light.current, lightTarget.current, width, height)
  }, [height, width])
  return <>
    <hemisphereLight args={[0xffe4bd, 0x26333a, 1.55]} />
    <object3D ref={lightTarget} position={lighting.target} />
    <directionalLight
      ref={light} color={0xffd5a0} intensity={2.2} position={lighting.position}
      castShadow shadow-mapSize-width={1024} shadow-mapSize-height={1024}
      shadow-camera-left={-10} shadow-camera-right={10}
      shadow-camera-top={10} shadow-camera-bottom={-10} shadow-camera-far={30}
      shadow-bias={-0.00015}
    />
    <Instances geometry={atlas.geometry('material.floor.top.0', 'center')} material={material} instances={model.floors[0]} />
    <Instances geometry={atlas.geometry('material.floor.top.1', 'center')} material={material} instances={model.floors[1]} />
    <Instances geometry={atlas.geometry('material.wall.top', 'center')} material={material} instances={model.walls.tops} castShadow />
    <Instances geometry={atlas.geometry('material.wall.side', 'center')} material={material} instances={model.walls.sides} castShadow />
    <Instances geometry={atlas.geometry('material.block.top', 'center')} material={material} instances={model.blocks.tops} castShadow />
    <Instances geometry={atlas.geometry('material.block.side', 'center')} material={material} instances={model.blocks.sides} castShadow />
    <Instances geometry={atlas.geometry('material.lair.top', 'center')} material={material} instances={model.lairs.tops} castShadow />
    <Instances geometry={atlas.geometry('material.lair.side', 'center')} material={material} instances={model.lairs.sides} castShadow />
    <Instances geometry={atlas.geometry('terrain.lair', 'center')} material={material} instances={model.lairs.entrances} />
    <Instances geometry={atlas.geometry('material.rim.top', 'center')} material={material} instances={model.rim.tops} castShadow />
    <Instances geometry={atlas.geometry('material.rim.side', 'center')} material={material} instances={model.rim.sides} castShadow />
    {children}
  </>
}
