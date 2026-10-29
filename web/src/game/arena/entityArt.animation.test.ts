import { describe, expect, it, vi } from 'vitest'
import { makeBomb } from './entityArt'

vi.mock('pixi.js', () => {
  class Node {
    children: Node[] = []
    visible = true
    alpha = 1
    addChild(...children: Node[]): Node { this.children.push(...children); return children[0] }
  }
  class Graphics extends Node {
    rectangles: Array<[number, number, number, number, number]> = []
    private pending: [number, number, number, number] | null = null
    clear(): this { this.rectangles = []; return this }
    rect(x: number, y: number, width: number, height: number): this {
      this.pending = [x, y, width, height]
      return this
    }
    fill(color: number): this {
      if (!this.pending) throw new Error('fill must follow a rectangle')
      this.rectangles.push([...this.pending, color])
      this.pending = null
      return this
    }
  }
  return { Container: Node, Graphics }
})

describe('native bomb shell animation', () => {
  it('draws three distinct timed poses and leaves other triggers single-frame', () => {
    const timed = makeBomb('power', 'timed')
    const shells = timed.shellFrames?.map((frame) =>
      (frame as unknown as { rectangles: unknown[] }).rectangles)

    expect(shells).toHaveLength(3)
    expect(new Set(shells?.map((rectangles) => JSON.stringify(rectangles))).size).toBe(3)
    expect(timed.root.children.slice(0, 4)).toEqual([
      timed.shadow, timed.shell, timed.spark, timed.pause,
    ])
    expect(timed.root.children.slice(4)).toEqual(timed.shellFrames?.slice(1))
    expect(timed.shellFrames?.map(({ visible }) => visible)).toEqual([true, false, false])
    expect(makeBomb('power', 'remote').shellFrames).toBeUndefined()
    expect(makeBomb('standard', 'contact').shellFrames).toBeUndefined()
  })
})
