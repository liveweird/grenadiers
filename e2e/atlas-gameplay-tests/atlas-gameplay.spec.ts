import { expect, test } from '@playwright/test'
import { observeBrowserPolicy, type BrowserPolicyObservation } from '../tests/support/browser-policy'
import bombPulseFingerprints from './bomb-pulse-fingerprints'

let browserPolicy: BrowserPolicyObservation
test.beforeEach(async ({ context }) => { browserPolicy = await observeBrowserPolicy(context) })
test.afterEach(() => { expect(browserPolicy.violations).toEqual([]) })

type Result = {
  resolution: number
  comparisons: Array<{
    stage: string; differentBytes: number; differentPixels: number
    maxRgbaDelta: [number, number, number, number]; width: number; height: number
    bounds: { left: number; top: number; right: number; bottom: number } | null
    samples: Array<{ x: number; y: number; atlas: number[]; graphics: number[] }>
  }>
  historicalDeltas: Result['comparisons']
  bombFingerprints: Array<{ stage: string; atlas: string; historical: string }>
  animationCoverage: {
    actors: Array<{
      stage: string; appearance: string; direction: string; pose: number
      atlasVisible: number[]; referenceVisible: number[]; differentBytes: number
    }>
    bombs: Array<{
      stage: string; expectedPose: number
      atlasVisible: number[]; referenceVisible: number[]; differentBytes: number
    }>
  }
  lifecycle: Record<string, boolean>
  performance: { coldStartupMs: number; retainedFrames: number; retainedFramesMs: number }
  resources: {
    atlasNodes: number; atlasSprites: number; textureSources: number
    textureWidth: number; textureHeight: number; rgbaBytes: number
  }
}

const bombStages = [
  ...[180, 165, 120, 112, 60, 57, 3].map((ticks) => `bomb-fuse-${ticks}`),
  'bomb-paused', 'bomb-remote', 'bomb-contact',
  ...['standard', 'piercing', 'power'].map((blast) => `bomb-pulse-${blast}`),
  ...['paused', 'resumed', 'stale-render', 'reduced-motion', 'moved-same-id', 'terminal']
    .map((state) => `bomb-animation-${state}`),
]
const actorStages = (['north', 'east', 'south', 'west'] as const).flatMap((direction) =>
  [0, 2, 1, 3].map((pose) => `actor-walk-${direction}-${pose}`))
const staticActorStages = ['actor-reduced-motion', 'actor-paused', 'actor-non-running']

test('atlas gameplay matches procedural pixels and retains renderer resources through lifecycle changes',
  async ({ page }, testInfo) => {
    await page.goto('/atlas-gameplay.html')
    await expect(page.locator('body')).toHaveAttribute(
      'data-build-target', 'grenadiers-atlas-gameplay-diagnostic',
    )
    const result = await page.evaluate(async (): Promise<Result> => {
      const api = (window as typeof window & {
        grenadiersAtlasGameplay?: { run: () => Promise<Result> }
      }).grenadiersAtlasGameplay
      if (!api) throw new Error('Atlas gameplay diagnostic API is unavailable')
      return api.run()
    })
    await testInfo.attach('atlas-gameplay-result', {
      body: JSON.stringify(result), contentType: 'application/json',
    })
    const expectedResolution = Number(testInfo.project.name.at(-1))
    expect(result.resolution).toBe(expectedResolution)
    expect(result.bombFingerprints).toEqual(
      bombPulseFingerprints[String(expectedResolution) as keyof typeof bombPulseFingerprints],
    )
    expect(result.comparisons.map(({ stage }) => stage)).toEqual([
      'running', 'interpolated-fractional', 'animation-80ms', 'animation-90ms',
      'animation-110ms', 'reduced-motion', 'result-hidden-actor', 'reset',
      'changed-terrain', 'context-restored',
      ...actorStages,
      ...staticActorStages,
      ...bombStages,
      ...[
        'protection', 'shield', 'stopwatch', 'goldenFlame', 'shortFuse', 'shortFlame',
        'longFuse', 'reverseControls', 'diarrhea', 'cloak',
      ].flatMap((cue) => [`cue-${cue}-low`, `cue-${cue}-high`]),
    ])
    expect(result.comparisons.every(({ width, height }) =>
      width === 720 * expectedResolution && height === 624 * expectedResolution)).toBe(true)
    expect(result.comparisons.every(({ differentBytes }) => differentBytes === 0),
      JSON.stringify(result.comparisons)).toBe(true)
    expect(result.animationCoverage.actors).toHaveLength(76)
    expect(new Set(result.animationCoverage.actors.map(({ appearance }) => appearance)))
      .toEqual(new Set(['one', 'two', 'three', 'four']))
    expect(result.animationCoverage.actors.every(({ pose, atlasVisible, referenceVisible, differentBytes }) =>
      atlasVisible.length === 1 && atlasVisible[0] === pose
      && referenceVisible.length === 1 && referenceVisible[0] === pose
      && differentBytes === 0), JSON.stringify(result.animationCoverage.actors)).toBe(true)
    expect(new Set(result.animationCoverage.bombs.map(({ expectedPose }) => expectedPose)))
      .toEqual(new Set([0, 1, 2]))
    expect(result.animationCoverage.bombs.every(({
      expectedPose, atlasVisible, referenceVisible, differentBytes,
    }) => atlasVisible.length === 1 && atlasVisible[0] === expectedPose
      && referenceVisible.length === 1 && referenceVisible[0] === expectedPose
      && differentBytes === 0), JSON.stringify(result.animationCoverage.bombs)).toBe(true)
    const historicalByStage = new Map(result.historicalDeltas.map((value) => [value.stage, value]))
    expect(historicalByStage.get('running')?.differentPixels).toBe(105 * expectedResolution ** 2)
    expect(historicalByStage.get('animation-80ms')?.differentPixels)
      .toBe(15 * expectedResolution ** 2)
    expect(historicalByStage.get('animation-90ms')?.differentPixels)
      .toBe(15 * expectedResolution ** 2)
    expect(historicalByStage.get('animation-110ms')?.differentPixels)
      .toBe(105 * expectedResolution ** 2)
    const combinedEvidence = {
      running: [315, [63, 39, 38, 0]],
      'animation-80ms': [45, [36, 34, 39, 0]],
      'animation-90ms': [45, [36, 34, 39, 0]],
      'animation-110ms': [315, [63, 39, 43, 0]],
    } as const
    for (const [stage, [bytes, maxima]] of Object.entries(combinedEvidence)) {
      expect(historicalByStage.get(stage)?.differentBytes).toBe(bytes * expectedResolution ** 2)
      expect(historicalByStage.get(stage)?.maxRgbaDelta).toEqual(maxima)
    }
    const isolated = {
      stopwatch: [87, 255, [46, 44, 33, 0], [67, 38, 76, 50]],
      goldenFlame: [75, 215, [67, 44, 38, 0], [65, 37, 78, 47]],
      shortFuse: [15, 45, [38, 37, 27, 0], [92, 51, 97, 58]],
      shortFlame: [43, 129, [52, 50, 38, 0], [50, 48, 55, 55]],
      longFuse: [5, 15, [38, 37, 27, 0], [60, 54, 63, 61]],
      reverseControls: [114, 342, [42, 39, 29, 0], [43, 62, 100, 82]],
      diarrhea: [51, 153, [44, 41, 32, 0], [62, 43, 83, 51]],
    } as const
    for (const [cue, evidence] of Object.entries(isolated)) {
      const delta = historicalByStage.get(`cue-${cue}-low`)
      expect(delta?.differentPixels, JSON.stringify(delta))
        .toBe(evidence[0] * expectedResolution ** 2)
      expect(delta?.differentBytes).toBe(evidence[1] * expectedResolution ** 2)
      expect(delta?.maxRgbaDelta).toEqual(evidence[2])
      const bounds = delta?.bounds
      expect(bounds && [
        bounds.left / expectedResolution, bounds.top / expectedResolution,
        (bounds.right + 1) / expectedResolution - 1,
        (bounds.bottom + 1) / expectedResolution - 1,
      ]).toEqual(evidence[3])
      expect(delta?.samples.length).toBeGreaterThan(0)
    }
    const intentionalRawDeltas = [
      'running', 'animation-80ms', 'animation-90ms', 'animation-110ms',
      'cue-stopwatch-low', 'cue-goldenFlame-low', 'cue-shortFuse-low',
      'cue-shortFlame-low', 'cue-longFuse-low', 'cue-reverseControls-low',
      'cue-diarrhea-low',
      ...bombStages,
      ...actorStages,
      ...staticActorStages,
    ]
    expect(result.historicalDeltas.filter(({ stage }) => !intentionalRawDeltas.includes(stage))
      .every(({ differentBytes }) => differentBytes === 0), JSON.stringify(result.historicalDeltas))
      .toBe(true)
    expect(Object.values(result.lifecycle).every(Boolean)).toBe(true)
    expect(result.resources.atlasSprites).toBeGreaterThan(195)
    expect(result.resources.atlasNodes).toBeLessThan(400)
    expect(result.resources).toMatchObject({
      textureSources: 1, textureWidth: 2040, textureHeight: 584, rgbaBytes: 4_765_440,
    })
    expect(result.performance.retainedFrames).toBe(30)
    expect(result.performance.coldStartupMs).toBeLessThan(10_000)
    expect(result.performance.retainedFramesMs).toBeLessThan(2_000)
  })
