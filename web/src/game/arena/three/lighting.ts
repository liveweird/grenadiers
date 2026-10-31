import { DirectionalLight, Object3D, Vector3 } from 'three'

export interface ArenaLightPlacement {
  position: [number, number, number]
  target: [number, number, number]
}

export function arenaLightPlacement(width: number, height: number): ArenaLightPlacement {
  const centerX = width / 2
  const centerZ = height / 2
  return {
    position: [centerX - 5.5, 9, centerZ + 6.5],
    target: [centerX, 0, centerZ],
  }
}

/** DirectionalLight only tracks a target whose world matrix is updated in the scene graph. */
export function aimArenaLight(
  light: DirectionalLight, target: Object3D, width: number, height: number,
): void {
  const placement = arenaLightPlacement(width, height)
  light.position.set(...placement.position)
  target.position.set(...placement.target)
  light.target = target
  target.updateMatrixWorld(true)
  light.updateMatrixWorld(true)
}

export function arenaLightDirection(
  light: DirectionalLight, output = new Vector3(),
): Vector3 {
  return output.setFromMatrixPosition(light.target.matrixWorld)
    .sub(new Vector3().setFromMatrixPosition(light.matrixWorld)).normalize()
}
