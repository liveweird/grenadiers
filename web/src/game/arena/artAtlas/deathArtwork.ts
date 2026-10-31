import type { Container } from 'pixi.js'

export type DeathFrame = 0 | 1 | 2 | 3

const DEATH_POSES = [
  { x: -2, y: 1, rotation: -0.18, scale: 1 },
  { x: 1, y: 5, rotation: 0.38, scale: 0.96 },
  { x: 2, y: 11, rotation: 1, scale: 0.9 },
  { x: 0, y: 16, rotation: Math.PI / 2, scale: 0.84 },
] as const

/** Procedural authoring reference; gameplay imports the adopted death frames from the atlas. */
export function poseDeathArtwork(drawing: Container, frame: DeathFrame): Container {
  const pose = DEATH_POSES[frame]
  drawing.position.set(pose.x, pose.y)
  drawing.rotation = pose.rotation
  drawing.scale.set(pose.scale)
  return drawing
}
