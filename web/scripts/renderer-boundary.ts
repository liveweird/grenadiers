import type { Plugin } from 'vite'

const referenceModules = /\/game\/arena\/(?:pixiRenderer|pixiCompatibility|graphicsArtwork|actorArt|drawing|terrainArt|entityArt|lairArt|weatherArt|artAtlas\/catalog)\.[cm]?[jt]sx?$/

export function assertProductionArenaModules(moduleIds: readonly string[]): void {
  const normalized = moduleIds.map((id) => id.replaceAll('\\', '/').split('?')[0])
  const forbidden = normalized.filter((id) =>
    /\/node_modules\/(?:pixi\.js|@pixi\/)/.test(id) || referenceModules.test(id))
  if (forbidden.length) throw new Error(`Reference renderer entered the production bundle:\n${forbidden.join('\n')}`)
  if (!normalized.some((id) => id.includes('/node_modules/three/'))
    || !normalized.some((id) => id.includes('/node_modules/@react-three/fiber/'))) {
    throw new Error('Production arena must include Three.js and React Three Fiber')
  }
}

/** Check the emitted module graph; minified string searches cannot prove engine separation. */
export function productionArenaBoundary(): Plugin {
  return {
    name: 'production-arena-boundary',
    apply: 'build',
    generateBundle(_options, bundle) {
      const moduleIds = Object.values(bundle).flatMap((output) => output.type === 'chunk'
        ? Object.entries(output.modules).filter(([, info]) => info.renderedLength > 0).map(([id]) => id)
        : [])
      assertProductionArenaModules(moduleIds)
      this.info('Production arena module graph contains Three/R3F and excludes Pixi reference code')
    },
  }
}
