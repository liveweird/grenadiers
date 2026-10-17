import { Container, Graphics } from 'pixi.js'
import { headingRotation, makeActor, makeBlock, makeBomb, makeFlame, paintTerrain, TILE } from './drawing'
import type { ActorDrawing } from './drawing'
import type { SnapshotMessage } from './types'

type Cell = { column: number; row: number }
type Motion = { fromX: number; fromY: number; toX: number; toY: number }
const key = (cell: Cell): string => `${cell.column},${cell.row}`
const unitsToPixels = (units: number): number => units * TILE / 1024

export class ArenaScene {
  readonly root = new Container()
  private readonly terrain = new Graphics()
  private readonly blocksLayer = new Container()
  private readonly bombsLayer = new Container()
  private readonly flamesLayer = new Container()
  private readonly actorsLayer = new Container()
  private readonly blocks = new Map<string, Graphics>()
  private readonly bombs = new Map<number, Container>()
  private readonly flames = new Map<string, Graphics>()
  private readonly actors = new Map<number, ActorDrawing>()
  private readonly motions = new Map<number, Motion>()
  private identity = ''
  private terrainKey = ''
  private receivedAt = 0
  private running = false
  private reducedMotion = false

  constructor() {
    this.root.addChild(this.terrain, this.blocksLayer, this.bombsLayer, this.flamesLayer, this.actorsLayer)
  }

  update(snapshot: SnapshotMessage, now: number): void {
    const identity = `${snapshot.room.roomId}:${snapshot.room.roundId}`
    const reset = identity !== this.identity
    if (reset) this.clearLayers()
    this.identity = identity
    this.receivedAt = now
    this.running = snapshot.room.phase.type === 'running'
    const terrainKey = snapshot.state.arena.terrain.join(',')
    if (terrainKey !== this.terrainKey) {
      paintTerrain(this.terrain, snapshot.state.arena)
      this.terrainKey = terrainKey
    }
    this.updateBlocks(snapshot.state.arena.blocks)
    this.updateBombs(snapshot.state.bombs)
    this.updateFlames(snapshot.state.flames)
    this.updateActors(snapshot.state.actors, reset)
    this.render(now)
  }

  setReducedMotion(reduced: boolean): void { this.reducedMotion = reduced }

  render(now: number): void {
    const progress = this.reducedMotion || !this.running ? 1
      : Math.max(0, Math.min(1, (now - this.receivedAt) / 50))
    for (const [id, drawing] of this.actors) {
      const motion = this.motions.get(id)!
      drawing.root.position.set(
        motion.fromX + (motion.toX - motion.fromX) * progress,
        motion.fromY + (motion.toY - motion.fromY) * progress,
      )
    }
    const effects = this.running && !this.reducedMotion
    for (const [id, bomb] of this.bombs) {
      bomb.scale.set(effects ? 1 + Math.sin(now / 160 + id) * 0.025 : 1)
    }
    for (const flame of this.flames.values()) {
      flame.alpha = effects ? 0.93 + Math.sin(now / 70) * 0.07 : 1
    }
  }

  private updateBlocks(cells: Cell[]): void {
    const present = new Set(cells.map(key))
    for (const [id, drawing] of this.blocks) {
      if (!present.has(id)) { drawing.destroy(); this.blocks.delete(id) }
    }
    for (const cell of cells) {
      if (this.blocks.has(key(cell))) continue
      const drawing = makeBlock()
      drawing.position.set(cell.column * TILE, cell.row * TILE)
      this.blocksLayer.addChild(drawing)
      this.blocks.set(key(cell), drawing)
    }
  }

  private updateBombs(rows: SnapshotMessage['state']['bombs']): void {
    const present = new Set(rows.map((bomb) => bomb.id))
    for (const [id, drawing] of this.bombs) {
      if (!present.has(id)) { drawing.destroy({ children: true, context: true }); this.bombs.delete(id) }
    }
    for (const bomb of rows) {
      if (this.bombs.has(bomb.id)) continue
      const drawing = makeBomb()
      drawing.position.set((bomb.cell.column + 0.5) * TILE, (bomb.cell.row + 0.5) * TILE)
      this.bombsLayer.addChild(drawing)
      this.bombs.set(bomb.id, drawing)
    }
  }

  private updateFlames(rows: SnapshotMessage['state']['flames']): void {
    const present = new Set(rows.map((flame) => key(flame.cell)))
    for (const [id, drawing] of this.flames) {
      if (!present.has(id)) { drawing.destroy(); this.flames.delete(id) }
    }
    for (const flame of rows) {
      if (this.flames.has(key(flame.cell))) continue
      const drawing = makeFlame()
      drawing.position.set(flame.cell.column * TILE, flame.cell.row * TILE)
      this.flamesLayer.addChild(drawing)
      this.flames.set(key(flame.cell), drawing)
    }
  }

  private updateActors(rows: SnapshotMessage['state']['actors'], reset: boolean): void {
    const present = new Set(rows.map((actor) => actor.id))
    for (const [id, drawing] of this.actors) {
      if (!present.has(id)) { drawing.root.destroy({ children: true, context: true }); this.actors.delete(id); this.motions.delete(id) }
    }
    for (const actor of rows) {
      let drawing = this.actors.get(actor.id)
      const toX = unitsToPixels(actor.position.x)
      const toY = unitsToPixels(actor.position.y)
      if (!drawing) {
        drawing = makeActor(actor.seat)
        drawing.root.position.set(toX, toY)
        this.actorsLayer.addChild(drawing.root)
        this.actors.set(actor.id, drawing)
      }
      this.motions.set(actor.id, {
        fromX: reset ? toX : drawing.root.x, fromY: reset ? toY : drawing.root.y, toX, toY,
      })
      drawing.root.visible = actor.alive
      drawing.body.rotation = headingRotation(actor.heading)
    }
  }

  private clearLayers(): void {
    for (const drawing of this.blocks.values()) drawing.destroy()
    for (const drawing of this.bombs.values()) drawing.destroy({ children: true, context: true })
    for (const drawing of this.flames.values()) drawing.destroy()
    for (const drawing of this.actors.values()) drawing.root.destroy({ children: true, context: true })
    this.blocks.clear(); this.bombs.clear(); this.flames.clear(); this.actors.clear(); this.motions.clear()
    this.terrainKey = ''
  }
}
