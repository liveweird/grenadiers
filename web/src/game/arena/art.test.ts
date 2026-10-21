import { describe, expect, it, vi } from 'vitest'
import { makeActor, setActorPose } from './actorArt'
import { makeBomb, makeFlame, makePickup } from './entityArt'
import { PALETTE } from './palette'
import { deterministicVariant } from './pixel'
import { makeBlock, paintTerrain, TILE } from './terrainArt'

vi.mock('pixi.js', () => {
  class Node {
    children: Node[] = []
    visible = true
    alpha = 1
    addChild(...children: Node[]): Node { this.children.push(...children); return children[0] }
  }
  class Graphics extends Node {
    rectangles: Array<{ x: number; y: number; width: number; height: number; color: number }> = []
    private pending: [number, number, number, number] | null = null
    clear(): this { this.rectangles = []; return this }
    rect(x: number, y: number, width: number, height: number): this {
      this.pending = [x, y, width, height]
      return this
    }
    fill(color: number): this {
      if (!this.pending) throw new Error('fill must follow a pixel rectangle')
      const [x, y, width, height] = this.pending
      this.rectangles.push({ x, y, width, height, color })
      this.pending = null
      return this
    }
  }
  return { Container: Node, Graphics }
})

type Drawing = {
  children: Drawing[]
  visible: boolean
  rectangles?: Array<{ x: number; y: number; width: number; height: number; color: number }>
}

function descendants(root: Drawing): Drawing[] {
  return [root, ...root.children.flatMap(descendants)]
}

describe('original pixel-art geometry', () => {
  it('uses deterministic coordinate variants for repeatable public terrain detail', () => {
    for (let row = 0; row < 13; row += 1) {
      for (let column = 0; column < 15; column += 1) {
        const variant = deterministicVariant(column, row, 4)
        expect(variant).toBeGreaterThanOrEqual(0)
        expect(variant).toBeLessThan(4)
        expect(deterministicVariant(column, row, 4)).toBe(variant)
      }
    }
    const drawing = makeBlock() as unknown as Drawing
    expect(drawing.rectangles).toHaveLength(18)
    expect(drawing.rectangles?.every(({ width, height }) => width > 0 && height > 0)).toBe(true)
  })

  it('builds varied floor and masonry entirely from crisp integer rectangles', () => {
    const Graphics = makeBlock().constructor as new () => Parameters<typeof paintTerrain>[0]
    const terrain = new Graphics() as unknown as Drawing
    const tiles: Array<'floor' | 'wall'> = Array.from({ length: 15 * 13 }, () => 'floor')
    tiles[2] = 'wall'
    paintTerrain(terrain as never, { width: 15, height: 13, terrain: tiles, blocks: [] })
    const rectangles = terrain.rectangles ?? []
    expect(rectangles.length).toBeGreaterThan(30)
    expect(rectangles.every(({ x, y, width, height }) =>
      [x, y, width, height].every(Number.isInteger) && width > 0 && height > 0)).toBe(true)
    const first = rectangles.filter(({ x }) => x < TILE).map(({ x, y }) => [x, y])
    const second = rectangles.filter(({ x }) => x >= TILE && x < TILE * 2)
      .map(({ x, y }) => [x - TILE, y])
    expect(second).not.toEqual(first)
    expect(rectangles.some(({ color }) => color === PALETTE.moss)).toBe(true)
  })

  it('gives both uniforms eight bounded directional walking poses with one visible frame', () => {
    const gold = makeActor('one')
    const blue = makeActor('two')
    for (const actor of [gold, blue]) {
      expect(actor.root.children).toHaveLength(26)
      expect(Object.values(actor.poses).flat()).toHaveLength(8)
      expect(Object.values(actor.poses).flat().filter(({ visible }) => visible)).toHaveLength(1)
      setActorPose(actor, 'north', 1)
      expect(actor.poses.north.map(({ visible }) => visible)).toEqual([false, true])
      expect(Object.values(actor.poses).flat().filter(({ visible }) => visible)).toHaveLength(1)
    }
    const goldColors = descendants(gold.root as unknown as Drawing)
      .flatMap(({ rectangles = [] }) => rectangles.map(({ color }) => color))
    const blueColors = descendants(blue.root as unknown as Drawing)
      .flatMap(({ rectangles = [] }) => rectangles.map(({ color }) => color))
    expect(goldColors).toContain(PALETTE.brass)
    expect(blueColors).toContain(PALETTE.blue)
  })

  it('keeps hazards and beveled pickups detailed with a fixed resource bound', () => {
    const bomb = makeBomb()
    const flame = makeFlame()
    expect(bomb.root.children).toHaveLength(4)
    expect(flame.root.children).toHaveLength(2)
    expect(flame.frames.map(({ visible }) => visible)).toEqual([true, false])
    const pickups = [
      'bombCapacity', 'blastRange', 'movementSpeed', 'kick', 'wallPass', 'heart', 'remoteBomb', 'piercingBomb', 'powerBomb', 'landmine', 'shield', 'stopwatch', 'invisibilityCloak', 'goldenFlame', 'shortFuse',
      'diarrhea', 'slowFoot', 'shortFlame', 'longFuse', 'reverseControls', 'invisibleBombs', 'teleporter',
    ] as const
    const counts = pickups.map((kind) => makePickup(kind) as unknown as Drawing)
      .map(({ rectangles = [] }) => rectangles.length)
    expect(counts).toEqual([16, 14, 18, 15, 19, 23, 19, 17, 19, 16, 21, 15, 21, 22, 21, 20, 18, 22, 20, 21, 24, 24])
    expect(Math.max(...counts)).toBeLessThanOrEqual(24)
    const wallPass = makePickup('wallPass') as unknown as Drawing
    const wallPassColors = wallPass.rectangles?.map(({ color }) => color)
    expect(wallPassColors).toContain(PALETTE.brick)
    expect(wallPassColors).toContain(PALETTE.blueLight)
    const heart = makePickup('heart') as unknown as Drawing
    const colors = heart.rectangles?.map(({ color }) => color)
    expect(colors).toContain(PALETTE.brick)
    expect(colors).toContain(PALETTE.brickLight)
    const shield = makePickup('shield') as unknown as Drawing
    const shieldColors = shield.rectangles?.map(({ color }) => color)
    expect(shieldColors).toContain(PALETTE.blue)
    expect(shieldColors).toContain(PALETTE.blueLight)
    const cloak = makePickup('invisibilityCloak') as unknown as Drawing
    const cloakColors = cloak.rectangles?.map(({ color }) => color)
    expect(cloakColors).toContain(PALETTE.deepInk)
    expect(cloakColors).toContain(PALETTE.blue)
    const golden = makePickup('goldenFlame') as unknown as Drawing
    const goldenColors = golden.rectangles?.map(({ color }) => color)
    expect(goldenColors).toContain(PALETTE.flame)
    expect(goldenColors).toContain(PALETTE.flameCore)
    const shortFuse = makePickup('shortFuse') as unknown as Drawing
    const shortFuseColors = shortFuse.rectangles?.map(({ color }) => color)
    expect(shortFuseColors).toContain(PALETTE.ink)
    expect(shortFuseColors).toContain(PALETTE.flameCore)
    const diarrhea = makePickup('diarrhea') as unknown as Drawing
    const diarrheaColors = diarrhea.rectangles?.map(({ color }) => color)
    expect(diarrheaColors).toContain(PALETTE.ink)
    expect(diarrheaColors).toContain(PALETTE.brick)
    const shortFlame = makePickup('shortFlame') as unknown as Drawing
    const shortFlameColors = shortFlame.rectangles?.map(({ color }) => color)
    expect(shortFlameColors).toContain(PALETTE.blue)
    expect(shortFlameColors).toContain(PALETTE.brick)
    const longFuse = makePickup('longFuse') as unknown as Drawing
    const longFuseColors = longFuse.rectangles?.map(({ color }) => color)
    expect(longFuseColors).toContain(PALETTE.ink)
    expect(longFuseColors).toContain(PALETTE.flameCore)
    const reverse = makePickup('reverseControls') as unknown as Drawing
    const reverseColors = reverse.rectangles?.map(({ color }) => color)
    expect(reverseColors).toContain(PALETTE.white)
    expect(reverseColors).toContain(PALETTE.brick)
  })

  it('gives Piercing pickups, held charges, and captured bombs distinct pixel cues', () => {
    const pickup = makePickup('piercingBomb') as unknown as Drawing
    const pickupColors = pickup.rectangles?.map(({ color }) => color)
    expect(pickupColors).toContain(PALETTE.white)
    expect(pickupColors).toContain(PALETTE.brick)
    const actor = makeActor('one')
    expect((actor.piercing as unknown as Drawing).rectangles?.length).toBeGreaterThan(0)
    const standard = makeBomb('standard').root as unknown as Drawing
    const piercing = makeBomb('piercing').root as unknown as Drawing
    expect(descendants(piercing).flatMap(({ rectangles = [] }) => rectangles))
      .not.toEqual(descendants(standard).flatMap(({ rectangles = [] }) => rectangles))
  })

  it('gives Power pickups, held charges, and captured bombs distinct diamond cues', () => {
    const pickup = makePickup('powerBomb') as unknown as Drawing
    expect(pickup.rectangles?.map(({ color }) => color)).toContain(PALETTE.flame)
    const actor = makeActor('one')
    expect((actor.power as unknown as Drawing).rectangles?.length).toBeGreaterThan(0)
    const standard = makeBomb('standard').root as unknown as Drawing
    const power = makeBomb('power').root as unknown as Drawing
    expect(descendants(power).flatMap(({ rectangles = [] }) => rectangles))
      .not.toEqual(descendants(standard).flatMap(({ rectangles = [] }) => rectangles))
  })

  it('gives Remote pickups and armed bombs an antenna without a burning fuse', () => {
    const pickup = makePickup('remoteBomb') as unknown as Drawing
    expect(pickup.rectangles?.map(({ color }) => color)).toContain(PALETTE.brass)
    expect(pickup.rectangles?.map(({ color }) => color)).toContain(PALETTE.brick)
    const timed = makeBomb('standard', 'timed')
    const remote = makeBomb('standard', 'remote')
    expect(remote.remote).toBe(true)
    expect(remote.spark.visible).toBe(false)
    expect(descendants(remote.root as unknown as Drawing).flatMap(({ rectangles = [] }) => rectangles))
      .not.toEqual(descendants(timed.root as unknown as Drawing).flatMap(({ rectangles = [] }) => rectangles))
  })

  it('gives Landmine pickups and placed mines a distinct pressure plate with no spark', () => {
    const pickup = makePickup('landmine') as unknown as Drawing
    expect(pickup.rectangles?.map(({ color }) => color)).toContain(PALETTE.brass)
    const timed = makeBomb('standard', 'timed')
    const mine = makeBomb('standard', 'contact')
    expect(mine.contact).toBe(true)
    expect(mine.spark.visible).toBe(false)
    expect(descendants(mine.root as unknown as Drawing).flatMap(({ rectangles = [] }) => rectangles))
      .not.toEqual(descendants(timed.root as unknown as Drawing).flatMap(({ rectangles = [] }) => rectangles))
  })
})
