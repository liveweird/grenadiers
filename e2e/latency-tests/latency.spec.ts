import { performance } from 'node:perf_hooks'
import { expect, test, type Browser, type BrowserContext, type Page, type TestInfo } from '@playwright/test'
import {
  installDelayedWebSocket, type DelayBridgeHandle, type RoutedFrameSummary,
} from '../tests/support/delayed-websocket'
import { EventLoopDelayMonitor } from '../tests/support/event-loop-diagnostics'
import {
  distribution, LatencyObserver, sample, type LatencySample, type TrajectoryChoice,
} from '../tests/support/latency-observer'
import {
  domClockDiagnostics, installDomKeyTimings, type DomClockDiagnostics,
} from '../tests/support/page-latency-diagnostics'

type Profile = { name: 'local' | 'delay-50ms'; baseMs: number; jitterMs: readonly number[]; budget: boolean }
const LOCAL: Profile = { name: 'local', baseMs: 0, jitterMs: [0], budget: false }
const SHAPED: Profile = { name: 'delay-50ms', baseMs: 50, jitterMs: [-10, 0, 10, 0], budget: true }
const BUDGET_MS = 250

test.beforeAll(() => {
  expect(process.env.E2E_MATCH_FIXTURE, 'Configure the documented match fixture for latency acceptance')
    .toBe('arena-v1-matches')
})

async function joinRunning(pages: Page[]): Promise<void> {
  const [one, two] = pages
  const baseUrl = process.env.E2E_BASE_URL ?? 'http://127.0.0.1:8085'
  await Promise.all(pages.map((page) => page.goto(new URL('/', baseUrl).href)))
  await one.getByRole('button', { name: 'Create room', exact: true }).click()
  const code = one.getByLabel('Room invitation code', { exact: true })
  await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await code.innerText())
  await two.getByRole('button', { name: 'Join room', exact: true }).click()
  await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Ready', exact: true }).click()))
  await Promise.all(pages.map((page) => expect(page.getByText('Round 1', { exact: true })).toBeVisible()))
}

async function observeMovement(
  page: Page,
  observer: LatencyObserver,
  direction: 'east' | 'west',
): Promise<LatencySample[]> {
  const key = direction === 'east' ? 'ArrowRight' : 'ArrowLeft'
  const baseline = observer.latestSnapshot('peer')
  const actionIndex = observer.forwardedFrames.length
  const actionStart = performance.now()
  await page.keyboard.down(key)
  const actionFinished = performance.now()
  const outgoing = await observer.forwardedMovement('actor', actionIndex, direction)
  const [moved, movingWitness] = await observer.twoMoved('peer', outgoing.at, baseline, direction)
  const action = sample({
    kind: 'action', direction, keyStart: actionStart, keyFinished: actionFinished,
    outgoingAt: outgoing.at, observedAt: moved.at,
  })

  const releaseIndex = observer.forwardedFrames.length
  const releaseStart = performance.now()
  await page.keyboard.up(key)
  const releaseFinished = performance.now()
  const stoppedOutgoing = await observer.forwardedMovement('actor', releaseIndex, null)
  const stable = await observer.firstStablePair(
    'peer', stoppedOutgoing.at, movingWitness.revision, direction,
  )
  const release = sample({
    kind: 'release', direction: null, keyStart: releaseStart, keyFinished: releaseFinished,
    outgoingAt: stoppedOutgoing.at, observedAt: stable.observed.at, confirmedAt: stable.confirmed.at,
  })
  return [action, release]
}

function publicFrames(frames: RoutedFrameSummary[]) {
  return frames.map((frame) => ({
    endpoint: frame.endpoint, direction: frame.direction, at: frame.at,
    bytes: frame.bytes, type: frame.type, movement: frame.movement,
    movementSequence: frame.movementSequence, actionId: frame.actionId,
    snapshot: frame.snapshot,
    enqueueAt: frame.enqueueAt, dueAt: frame.dueAt,
    dispatchAt: frame.dispatchAt, timerLatenessMs: frame.timerLatenessMs,
  }))
}

type ReportInput = {
  testInfo: TestInfo
  profile: Profile
  observer: LatencyObserver
  samples: LatencySample[]
  domClock: DomClockDiagnostics | undefined
  eventLoop: EventLoopDelayMonitor
  handles: DelayBridgeHandle[]
  trajectory: TrajectoryChoice[]
}

async function attachReport(input: ReportInput): Promise<void> {
  const { testInfo, profile, observer, samples, domClock, eventLoop, handles, trajectory } = input
  const exceedances = samples.filter((value) => value.latencyMs > BUDGET_MS)
  const summary = {
    profile: profile.name,
    distribution: distribution(samples),
    exceedanceCount: exceedances.length,
    samples: samples.map((value) => ({ kind: value.kind, latencyMs: value.latencyMs,
      routeAfterKeyMs: value.outgoingAt - value.keyStart,
      publicAfterRouteMs: value.observedAt - value.outgoingAt,
      confirmationLagMs: value.confirmedAt === undefined ? undefined : value.confirmedAt - value.observedAt })),
    trajectory,
    queuePeaks: handles.map((handle) => handle.summary()),
  }
  console.log(`LATENCY_PUBLIC_SUMMARY ${JSON.stringify(summary)}`)
  await testInfo.attach(`latency-${profile.name}-public-summary`, {
    contentType: 'application/json',
    body: JSON.stringify({
      profile: { baseMs: profile.baseMs, jitterMs: profile.jitterMs, direction: 'each-way' },
      budgetMs: BUDGET_MS,
      stableReleaseMethod: 'two page-delivered moving revisions, null server dispatch, then two page-delivered stable revisions whose route ingress followed dispatch',
      distribution: distribution(samples), samples, exceedances,
      trajectory,
      nodeEventLoop: eventLoop.summary(),
      domClock: {
        independentFromNodeClock: true,
        timeOrigin: domClock?.timeOrigin ?? null,
        timings: domClock?.timings ?? [],
      },
      pageMovementSends: observer.pageMovementSends,
      routedFrames: publicFrames(observer.frames),
      forwardedFrames: publicFrames(observer.forwardedFrames),
      publicSnapshots: observer.snapshots.slice(-512),
      publicArenas: Object.fromEntries(observer.arenas),
      queues: handles.map((handle) => handle.summary()),
      closedBridges: observer.bridges,
      failures: observer.failures,
    }),
  })
}

async function runProfile(browser: Browser, testInfo: TestInfo, profile: Profile): Promise<void> {
  const contexts: BrowserContext[] = []
  const observer = new LatencyObserver()
  const samples: LatencySample[] = []
  const handles: DelayBridgeHandle[] = []
  const trajectory: TrajectoryChoice[] = []
  const eventLoop = new EventLoopDelayMonitor()
  let domClock: DomClockDiagnostics | undefined
  let reportAttached = false
  try {
    contexts.push(await browser.newContext())
    contexts.push(await browser.newContext())
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    const [one, two] = pages
    handles.push(
      await installDelayedWebSocket(one, 'actor', profile, observer),
      await installDelayedWebSocket(two, 'peer', profile, observer),
    )
    await installDomKeyTimings(one)
    await observer.observePageSnapshots(one, 'actor')
    await observer.observePageSnapshots(two, 'peer')
    await joinRunning(pages)
    expect(observer.installedEndpoints).toEqual(new Set(['actor', 'peer']))
    expect(observer.pageSocketCounts).toEqual(new Map([['actor', 1], ['peer', 1]]))
    const arena = one.getByRole('region', { name: 'Grenadiers courtyard', exact: true })
    await arena.click()
    for (let index = 0; index < 10; index += 1) {
      const choice = observer.safestHorizontalDirection('peer')
      trajectory.push(choice)
      samples.push(...await observeMovement(one, observer, choice.direction))
    }
    domClock = await domClockDiagnostics(one)
    expect(domClock.timings.filter((timing) => timing.key === 'ArrowRight'
      || timing.key === 'ArrowLeft')).toHaveLength(20)

    const bombIndex = observer.forwardedFrames.length
    await one.keyboard.press('Space')
    await expect.poll(() => observer.forwardedFrames.slice(bombIndex).filter((frame) => frame.endpoint === 'actor'
      && frame.direction === 'clientToServer' && frame.type === 'manualAction').length).toBe(1)
    await expect.poll(() => observer.latestSnapshot('actor').bombIds.length).toBe(1)
    await expect.poll(() => observer.latestSnapshot('peer').bombIds).toEqual(observer.latestSnapshot('actor').bombIds)
    const bombCommands = observer.forwardedFrames.slice(bombIndex).filter((frame) => frame.endpoint === 'actor'
      && frame.direction === 'clientToServer' && frame.type === 'manualAction')
    expect(bombCommands.map((frame) => frame.actionId)).toEqual([1])
    expect(observer.latestSnapshot('peer')).toMatchObject({
      roundId: observer.latestSnapshot('actor').roundId,
      bombIds: observer.latestSnapshot('actor').bombIds,
    })

    handles.forEach((handle) => handle.close())
    await expect.poll(() => observer.bridges.length).toBe(2)
    expect(observer.failures).toEqual([])
    expect(observer.publicFactsAreMonotonic('actor')).toBe(true)
    expect(observer.publicFactsAreMonotonic('peer')).toBe(true)
    expect(observer.everyPageSnapshotFollowedRouteDispatch('actor')).toBe(true)
    expect(observer.everyPageSnapshotFollowedRouteDispatch('peer')).toBe(true)
    expect(samples).toHaveLength(20)
    const exceedances = samples.filter((value) => value.latencyMs > BUDGET_MS)
    await attachReport({ testInfo, profile, observer, samples, domClock, eventLoop, handles, trajectory })
    reportAttached = true
    if (profile.budget) expect(exceedances).toEqual([])
  } catch (error) {
    if (!reportAttached) {
      await attachReport({ testInfo, profile, observer, samples, domClock, eventLoop, handles, trajectory })
    }
    throw error
  } finally {
    handles.forEach((handle) => handle.close())
    await Promise.all(contexts.map((context) => context.close().catch(() => undefined)))
    eventLoop.stop()
  }
}

test('native controls publish twenty unshaped local latency observations', async ({ browser }, testInfo) => {
  await runProfile(browser, testInfo, LOCAL)
})

test('native controls stay within the selected shaped latency budget', async ({ browser }, testInfo) => {
  await runProfile(browser, testInfo, SHAPED)
})
