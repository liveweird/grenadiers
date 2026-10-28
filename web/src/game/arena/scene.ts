import { Container } from 'pixi.js'
import { setActorPose, type ActorDrawing } from './actorArt'
import type { ArenaArtwork } from './artwork'
import { selectBombAnimation } from './bombAnimation'
import { TILE } from './drawing'
import type { BombDrawing, FlameDrawing, OilBarrelDrawing } from './entityArt'
import { graphicsArtwork } from './graphicsArtwork'
import type { RenderablePickupKind, SnapshotMessage } from './types'

type Cell = { column: number; row: number }
type Motion = {
  fromX: number; fromY: number; toX: number; toY: number
  lastTeleportTick: number | null | undefined
}
type PickupDrawing = { kind: RenderablePickupKind; drawing: Container }
type ActorRow = SnapshotMessage['state']['actors'][number]
const key = (cell: Cell): string => `${cell.column},${cell.row}`
const unitsToPixels = (units: number): number => units * TILE / 1024
const cloakActive = (remaining: number | undefined): boolean => (remaining ?? 0) > 0
function renderCloak(drawing: ActorDrawing, effects: boolean, now: number): void {
  drawing.cloak.visible = drawing.cloaked
  drawing.cloak.alpha = effects && drawing.cloaked ? 0.7 + (Math.floor(now / 140) % 2) * 0.3 : 1
}
function renderActor(drawing: ActorDrawing, id: number, effects: boolean, now: number): void {
  const frame = effects && drawing.moving && (Math.floor(now / 110) + id) % 2 === 1 ? 1 : 0
  setActorPose(drawing, drawing.direction, frame)
  renderActorProtection(drawing, effects, now)
  renderActorModifiers(drawing, effects, now)
  renderActorEquipment(drawing)
}
function renderActorProtection(drawing: ActorDrawing, effects: boolean, now: number): void {
  drawing.protection.visible = drawing.protected
  drawing.protection.alpha = effects && drawing.protected ? 0.65 + (Math.floor(now / 120) % 2) * 0.35 : 1
  drawing.shield.visible = drawing.shielded
  drawing.shield.alpha = effects && drawing.shielded ? 0.75 + (Math.floor(now / 160) % 2) * 0.25 : 1
  drawing.stopwatch.visible = drawing.stopwatchActive
  drawing.stopwatch.alpha = effects && drawing.stopwatchActive
    ? 0.7 + (Math.floor(now / 120) % 2) * 0.3 : 1
  drawing.frozen.visible = drawing.actionsFrozen
  drawing.goldenFlame.visible = drawing.golden
  drawing.goldenFlame.alpha = effects && drawing.golden ? 0.7 + (Math.floor(now / 100) % 2) * 0.3 : 1
  renderCloak(drawing, effects, now)
}
function renderActorModifiers(drawing: ActorDrawing, effects: boolean, now: number): void {
  drawing.shortFuse.visible = drawing.shortFused
  drawing.shortFuse.alpha = effects && drawing.shortFused ? 0.65 + (Math.floor(now / 75) % 2) * 0.35 : 1
  drawing.shortFlame.visible = drawing.shortFlamed
  drawing.shortFlame.alpha = effects && drawing.shortFlamed ? 0.7 + (Math.floor(now / 130) % 2) * 0.3 : 1
  drawing.longFuse.visible = drawing.longFused
  drawing.longFuse.alpha = effects && drawing.longFused ? 0.65 + (Math.floor(now / 150) % 2) * 0.35 : 1
  drawing.reverseControls.visible = drawing.reversed
  drawing.reverseControls.alpha = effects && drawing.reversed ? 0.7 + (Math.floor(now / 140) % 2) * 0.3 : 1
  drawing.diarrhea.visible = drawing.diarrheaActive
  drawing.diarrhea.alpha = effects && drawing.diarrheaActive
    ? 0.65 + (Math.floor(now / 105) % 2) * 0.35 : 1
}
function renderActorEquipment(drawing: ActorDrawing): void {
  drawing.slowFoot.visible = drawing.slowFootStacks > 0
  drawing.slowFoot.children.slice(1).forEach((marker, index) => {
    marker.visible = index < drawing.slowFootStacks
  })
  drawing.piercing.visible = drawing.piercingArmed
  drawing.power.visible = drawing.powerArmed
  drawing.landmine.visible = drawing.landmineStored
  drawing.kick.visible = drawing.kickEquipped
  drawing.punch.visible = drawing.punchEquipped
  drawing.bracelet.visible = drawing.braceletEquipped
  drawing.equipment.visible = drawing.kick.visible || drawing.punch.visible || drawing.bracelet.visible
  drawing.wallPass.visible = drawing.wallPassEquipped
}

function updateActorState(drawing: ActorDrawing, actor: ActorRow): void {
  drawing.root.visible = actor.alive
  drawing.protected = (actor.damageGraceRemainingTicks ?? 0) > 0
  drawing.shielded = (actor.shieldRemainingTicks ?? 0) > 0
  drawing.stopwatchActive = (actor.stopwatchRemainingTicks ?? 0) > 0
  drawing.actionsFrozen = Boolean(actor.voluntaryActionsFrozen)
  drawing.cloaked = cloakActive(actor.invisibilityCloakRemainingTicks)
  drawing.golden = (actor.goldenFlameRemainingTicks ?? 0) > 0
  drawing.shortFused = (actor.shortFuseRemainingTicks ?? 0) > 0
  drawing.shortFlamed = (actor.shortFlameRemainingTicks ?? 0) > 0
  drawing.longFused = (actor.longFuseRemainingTicks ?? 0) > 0
  drawing.reversed = (actor.reverseControlsRemainingTicks ?? 0) > 0
  drawing.diarrheaActive = (actor.diarrheaRemainingTicks ?? 0) > 0
  drawing.slowFootStacks = actor.slowFootRemainingTicks?.length ?? 0
  drawing.piercingArmed = Boolean(actor.piercingArmed)
  drawing.powerArmed = Boolean(actor.powerArmed)
  drawing.landmineStored = Boolean(actor.landmineStored)
  drawing.kickEquipped = Boolean(actor.kickEquipped)
  drawing.punchEquipped = Boolean(actor.punchEquipped)
  drawing.braceletEquipped = Boolean(actor.braceletEquipped)
  drawing.wallPassEquipped = Boolean(actor.wallPassEquipped)
}

export class ArenaScene {
  readonly root = new Container()
  private readonly terrain
  private readonly terrainCacheOptions
  private readonly artwork: ArenaArtwork
  private readonly blocksLayer = new Container()
  private readonly pickupsLayer = new Container()
  private readonly bombsLayer = new Container()
  private readonly flamesLayer = new Container()
  private readonly actorsLayer = new Container()
  private readonly blocks = new Map<string, Container>()
  private readonly pickups = new Map<string, PickupDrawing>()
  private readonly bombs = new Map<number, BombDrawing>()
  private readonly flames = new Map<string, FlameDrawing>()
  private readonly oilBarrels = new Map<string, OilBarrelDrawing>()
  private readonly actors = new Map<number, ActorDrawing>()
  private readonly motions = new Map<number, Motion>()
  private identity = ''
  private terrainKey = ''
  private receivedAt = 0
  private physicalTick = -1
  private motionDuration = 50
  private running = false
  private reducedMotion = false

  constructor(resolution: number, artwork: ArenaArtwork = graphicsArtwork) {
    this.artwork = artwork
    this.terrain = artwork.makeTerrain()
    this.terrainCacheOptions = { resolution, antialias: false, scaleMode: 'nearest' as const }
    this.root.addChild(
      this.terrain.root, this.blocksLayer, this.pickupsLayer, this.bombsLayer, this.flamesLayer, this.actorsLayer,
    )
  }

  update(snapshot: SnapshotMessage, now: number): void {
    const identity = `${snapshot.room.roomId}:${snapshot.room.roundId}:${snapshot.room.presentationBaseline ?? 'legacy'}`
    const physicalTick = snapshot.state.physicalTick ?? snapshot.state.runningUpdates ?? 0
    const tickDelta = physicalTick - this.physicalTick
    const reset = identity !== this.identity || tickDelta < 0
    if (reset) this.reset()
    this.identity = identity
    this.receivedAt = now
    this.motionDuration = Math.max(1000 / 60, Math.min(100, tickDelta * 1000 / 60))
    this.physicalTick = physicalTick
    this.running = snapshot.room.phase.type === 'running'
    const terrainKey = [
      snapshot.state.arena.width, snapshot.state.arena.height, ...snapshot.state.arena.terrain,
    ].join(',')
    if (terrainKey !== this.terrainKey) {
      if (!this.terrain.root.isCachedAsTexture) this.terrain.root.cacheAsTexture(this.terrainCacheOptions)
      this.terrain.paint(snapshot.state.arena)
      this.terrain.root.updateCacheTexture()
      this.terrainKey = terrainKey
    }
    this.updateBlocks(snapshot.state.arena.blocks)
    this.updatePickups(snapshot.state.pickups)
    this.updateBombs(snapshot.state.bombs)
    const burningOil = this.updateOilBarrels(snapshot.state.oilBarrels ?? [])
    this.updateFlames(snapshot.state.flames, burningOil)
    this.updateActors(snapshot.state.actors, reset)
    this.render(now)
  }

  setReducedMotion(reduced: boolean): void { this.reducedMotion = reduced }

  reset(): void {
    for (const drawing of this.blocks.values()) drawing.destroy()
    for (const pickup of this.pickups.values()) pickup.drawing.destroy()
    for (const drawing of this.bombs.values()) drawing.root.destroy({ children: true, context: true })
    for (const drawing of this.flames.values()) drawing.root.destroy({ children: true, context: true })
    for (const drawing of this.oilBarrels.values()) drawing.root.destroy({ children: true, context: true })
    for (const drawing of this.actors.values()) drawing.root.destroy({ children: true, context: true })
    this.blocks.clear(); this.pickups.clear(); this.bombs.clear(); this.flames.clear()
    this.oilBarrels.clear(); this.actors.clear()
    this.motions.clear()
    if (this.terrain.root.isCachedAsTexture) this.terrain.root.cacheAsTexture(false)
    this.terrain.clear()
    this.identity = ''
    this.terrainKey = ''
    this.receivedAt = 0
    this.running = false
    this.physicalTick = -1
    this.motionDuration = 50
  }

  restoreContext(): void {
    if (this.terrainKey) this.terrain.root.updateCacheTexture()
  }

  render(now: number): void {
    const progress = this.reducedMotion || !this.running ? 1
      : Math.max(0, Math.min(1, (now - this.receivedAt) / this.motionDuration))
    for (const [id, drawing] of this.actors) {
      const motion = this.motions.get(id)!
      drawing.root.position.set(
        motion.fromX + (motion.toX - motion.fromX) * progress,
        motion.fromY + (motion.toY - motion.fromY) * progress,
      )
    }
    const effects = this.running && !this.reducedMotion
    for (const bomb of this.bombs.values()) {
      const animation = selectBombAnimation(
        bomb.remainingTicks,
        effects && !bomb.paused && !bomb.remote && !bomb.contact,
      )
      bomb.shell.scale.set(animation.shellScale)
      bomb.spark.visible = !bomb.remote && !bomb.contact && !bomb.paused
        && animation.sparkVisible
      bomb.pause.visible = bomb.paused
    }
    for (const flame of this.flames.values()) {
      const frame = effects ? Math.floor(now / 80) % 2 : 0
      flame.frames[0].visible = frame === 0
      flame.frames[1].visible = frame === 1
    }
    for (const barrel of this.oilBarrels.values()) {
      const frame = effects ? Math.floor(now / 80) % 2 : 0
      barrel.flame.frames[0].visible = barrel.state === 'burning' && frame === 0
      barrel.flame.frames[1].visible = barrel.state === 'burning' && frame === 1
    }
    for (const [id, actor] of this.actors) {
      renderActor(actor, id, effects, now)
    }
  }

  private updateBlocks(cells: Cell[]): void {
    const present = new Set(cells.map(key))
    for (const [id, drawing] of this.blocks) {
      if (!present.has(id)) { drawing.destroy(); this.blocks.delete(id) }
    }
    for (const cell of cells) {
      if (this.blocks.has(key(cell))) continue
      const drawing = this.artwork.makeBlock()
      drawing.position.set(cell.column * TILE, cell.row * TILE)
      this.blocksLayer.addChild(drawing)
      this.blocks.set(key(cell), drawing)
    }
  }

  private updateBombs(rows: SnapshotMessage['state']['bombs']): void {
    const present = new Set(rows.map((bomb) => bomb.id))
    for (const [id, drawing] of this.bombs) {
      if (!present.has(id)) { drawing.root.destroy({ children: true, context: true }); this.bombs.delete(id) }
    }
    for (const bomb of rows) {
      let drawing = this.bombs.get(bomb.id)
      if (!drawing) {
        drawing = this.artwork.makeBomb(bomb.blast, bomb.trigger?.type)
        drawing.root.position.set((bomb.cell.column + 0.5) * TILE, (bomb.cell.row + 0.5) * TILE)
        this.bombsLayer.addChild(drawing.root)
        this.bombs.set(bomb.id, drawing)
      }
      drawing.root.position.set((bomb.cell.column + 0.5) * TILE, (bomb.cell.row + 0.5) * TILE)
      drawing.paused = bomb.trigger?.type === 'timed' && bomb.trigger.paused
      drawing.remainingTicks = bomb.trigger?.type === 'timed' ? bomb.trigger.remainingTicks : null
    }
  }

  private updatePickups(rows: SnapshotMessage['state']['pickups']): void {
    const present = new Set(rows.map((pickup) => key(pickup.cell)))
    for (const [id, pickup] of this.pickups) {
      if (!present.has(id)) { pickup.drawing.destroy(); this.pickups.delete(id) }
    }
    for (const pickup of rows) {
      const id = key(pickup.cell)
      const current = this.pickups.get(id)
      if (current?.kind === pickup.kind) continue
      if (current) { current.drawing.destroy(); this.pickups.delete(id) }
      const drawing = this.artwork.makePickup(pickup.kind)
      drawing.position.set(pickup.cell.column * TILE, pickup.cell.row * TILE)
      this.pickupsLayer.addChild(drawing)
      this.pickups.set(id, { kind: pickup.kind, drawing })
    }
  }

  private updateFlames(rows: SnapshotMessage['state']['flames'], burningOil: ReadonlySet<string>): void {
    const visibleRows = rows.filter((flame) => !burningOil.has(key(flame.cell)))
    const present = new Set(visibleRows.map((flame) => key(flame.cell)))
    for (const [id, drawing] of this.flames) {
      if (!present.has(id)) { drawing.root.destroy({ children: true, context: true }); this.flames.delete(id) }
    }
    for (const flame of visibleRows) {
      if (this.flames.has(key(flame.cell))) continue
      const drawing = this.artwork.makeFlame()
      drawing.root.position.set(flame.cell.column * TILE, flame.cell.row * TILE)
      this.flamesLayer.addChild(drawing.root)
      this.flames.set(key(flame.cell), drawing)
    }
  }

  private updateOilBarrels(
    rows: NonNullable<SnapshotMessage['state']['oilBarrels']>,
  ): Set<string> {
    const present = new Set(rows.map((barrel) => key(barrel.cell)))
    for (const [id, drawing] of this.oilBarrels) {
      if (!present.has(id)) {
        drawing.root.destroy({ children: true, context: true })
        this.oilBarrels.delete(id)
      }
    }
    const burning = new Set<string>()
    for (const barrel of rows) {
      const id = key(barrel.cell)
      let drawing = this.oilBarrels.get(id)
      if (!drawing) {
        drawing = this.artwork.makeOilBarrel()
        drawing.root.position.set(barrel.cell.column * TILE, barrel.cell.row * TILE)
        this.flamesLayer.addChild(drawing.root)
        this.oilBarrels.set(id, drawing)
      }
      drawing.state = barrel.state
      drawing.unlit.visible = barrel.state === 'unlit'
      drawing.charred.visible = barrel.state === 'burning'
      drawing.flame.root.visible = barrel.state === 'burning'
      if (barrel.state === 'burning') burning.add(id)
    }
    return burning
  }

  private updateActors(rows: SnapshotMessage['state']['actors'], reset: boolean): void {
    const present = new Set(rows.map((actor) => actor.id))
    for (const [id, drawing] of this.actors) {
      if (!present.has(id)) { drawing.root.destroy({ children: true, context: true }); this.actors.delete(id); this.motions.delete(id) }
    }
    for (const actor of rows) {
      let drawing = this.actors.get(actor.id)
      const previousMotion = this.motions.get(actor.id)
      const toX = unitsToPixels(actor.position.x)
      const toY = unitsToPixels(actor.position.y)
      if (!drawing) {
        drawing = this.artwork.makeActor(actor.seat)
        drawing.root.position.set(toX, toY)
        this.actorsLayer.addChild(drawing.root)
        this.actors.set(actor.id, drawing)
      }
      const enteringFreeze = actor.voluntaryActionsFrozen && !drawing.actionsFrozen
      const teleported = previousMotion !== undefined
        && previousMotion.lastTeleportTick !== actor.lastTeleportTick
      if (teleported) drawing.root.position.set(toX, toY)
      this.motions.set(actor.id, {
        fromX: reset || enteringFreeze || teleported ? toX : drawing.root.x,
        fromY: reset || enteringFreeze || teleported ? toY : drawing.root.y,
        toX, toY, lastTeleportTick: actor.lastTeleportTick,
      })
      updateActorState(drawing, actor)
      drawing.moving = Boolean(!drawing.actionsFrozen && !reset && !teleported && previousMotion
        && (previousMotion.toX !== toX || previousMotion.toY !== toY))
      drawing.direction = actor.heading
    }
  }
}
