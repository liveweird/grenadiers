import { Container } from 'pixi.js'
import type { ArenaArtwork, MobDrawing } from './artwork'
import { TILE } from './drawing'
import type { Direction, SnapshotMessage } from './types'

type Motion = { fromX: number; fromY: number; toX: number; toY: number }
type MobState = { drawing: MobDrawing; motion: Motion; frozen: boolean }

const unitsToPixels = (units: number): number => units * TILE / 1024

function showHeading(drawing: MobDrawing, heading: Direction): void {
  for (const [direction, pose] of Object.entries(drawing.poses)) pose.visible = direction === heading
}

export class EnvironmentScene {
  private readonly artwork: ArenaArtwork
  private readonly root: Container
  private readonly lairsLayer: Container
  private readonly mobsLayer: Container
  private weatherLayer: Container | null = null
  private readonly lairs = new Map<number, Container>()
  private readonly mobs = new Map<number, MobState>()
  private cloud: Container | null = null
  private cloudMotion: Motion | null = null

  constructor(artwork: ArenaArtwork, root: Container, lairsLayer: Container, mobsLayer: Container) {
    this.artwork = artwork
    this.root = root
    this.lairsLayer = lairsLayer
    this.mobsLayer = mobsLayer
  }

  update(state: SnapshotMessage['state'], reset: boolean): void {
    this.updateLairs(state.lairs)
    this.updateMobs(state.mobs, reset)
    this.updateCloud(state.stormCloud ?? null, reset)
  }

  render(progress: number): void {
    for (const { drawing, motion } of this.mobs.values()) {
      drawing.root.position.set(
        motion.fromX + (motion.toX - motion.fromX) * progress,
        motion.fromY + (motion.toY - motion.fromY) * progress,
      )
    }
    if (this.cloud && this.cloudMotion) {
      this.cloud.position.set(
        this.cloudMotion.fromX + (this.cloudMotion.toX - this.cloudMotion.fromX) * progress,
        this.cloudMotion.fromY + (this.cloudMotion.toY - this.cloudMotion.fromY) * progress,
      )
    }
  }

  reset(): void {
    for (const drawing of this.lairs.values()) drawing.destroy({ children: true, context: true })
    for (const mob of this.mobs.values()) mob.drawing.root.destroy({ children: true, context: true })
    this.cloud?.destroy({ children: true, context: true })
    this.weatherLayer?.destroy({ children: true, context: true })
    this.lairs.clear()
    this.mobs.clear()
    this.cloud = null
    this.cloudMotion = null
    this.weatherLayer = null
  }

  private updateLairs(rows: NonNullable<SnapshotMessage['state']['lairs']>): void {
    const present = new Set(rows.map(({ id }) => id))
    for (const [id, drawing] of this.lairs) {
      if (!present.has(id)) { drawing.destroy({ children: true, context: true }); this.lairs.delete(id) }
    }
    for (const lair of rows) {
      let drawing = this.lairs.get(lair.id)
      if (!drawing) {
        drawing = this.artwork.makeLair()
        this.lairsLayer.addChild(drawing)
        this.lairs.set(lair.id, drawing)
      }
      drawing.position.set(lair.cell.column * TILE, lair.cell.row * TILE)
    }
  }

  private updateMobs(rows: NonNullable<SnapshotMessage['state']['mobs']>, reset: boolean): void {
    const present = new Set(rows.map(({ id }) => id))
    for (const [id, mob] of this.mobs) {
      if (!present.has(id)) {
        mob.drawing.root.destroy({ children: true, context: true })
        this.mobs.delete(id)
      }
    }
    for (const mob of rows) {
      const toX = unitsToPixels(mob.position.x)
      const toY = unitsToPixels(mob.position.y)
      let current = this.mobs.get(mob.id)
      if (!current) {
        const drawing = this.artwork.makeMob()
        drawing.root.position.set(toX, toY)
        this.mobsLayer.addChild(drawing.root)
        current = { drawing, motion: { fromX: toX, fromY: toY, toX, toY }, frozen: mob.frozen }
        this.mobs.set(mob.id, current)
      }
      const enteringFreeze = mob.frozen && !current.frozen
      current.motion = {
        fromX: reset || enteringFreeze ? toX : current.drawing.root.x,
        fromY: reset || enteringFreeze ? toY : current.drawing.root.y,
        toX, toY,
      }
      current.frozen = mob.frozen
      showHeading(current.drawing, mob.heading)
    }
  }

  private updateCloud(row: SnapshotMessage['state']['stormCloud'], reset: boolean): void {
    if (!row) {
      this.cloud?.destroy({ children: true, context: true })
      this.weatherLayer?.destroy({ children: true, context: true })
      this.cloud = null
      this.cloudMotion = null
      this.weatherLayer = null
      return
    }
    const toX = unitsToPixels(row.position.x)
    const toY = unitsToPixels(row.position.y)
    if (!this.cloud) {
      this.weatherLayer = new Container()
      this.cloud = this.artwork.makeStormCloud()
      this.cloud.position.set(toX, toY)
      this.weatherLayer.addChild(this.cloud)
      this.root.addChild(this.weatherLayer)
    }
    this.cloudMotion = {
      fromX: reset || !this.cloudMotion ? toX : this.cloud.x,
      fromY: reset || !this.cloudMotion ? toY : this.cloud.y,
      toX, toY,
    }
  }
}
