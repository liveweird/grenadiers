import { expect, type Page, type TestInfo } from '@playwright/test'
import { CaptureBrowserTrace, sanitizeTraceEventsForTest } from '../tests/support/capture-browser-trace'
import { CaptureWorkloadProfile } from '../tests/support/capture-workload-profile'
import {
  installRenderCounters, readRenderCounters, removeRenderCounters, resetRenderCounters,
  stopAndReadRenderCounters,
} from '../tests/support/page-render-counters'

async function exerciseTraceSanitizer(): Promise<void> {
  const secret = 'fixed-trace-secret'
  const values: Array<Record<string, unknown>> = []
  for (let index = 0; index < 520; index += 1) values.push({
    name: 'Display::DrawAndSwap', cat: 'viz', ph: 'X', ts: index === 1 ? 600 : index,
    dur: index === 0 ? 999 : 4, id: index, id2: index === 0 ? secret : index + 1_000,
    pid: 10, tid: 11, args: { token: secret }, stack: secret,
  })
  values.push({
    name: secret, cat: 'network', ph: 'X', ts: 1, pid: 2, tid: 3, args: { url: secret },
  })
  values.push({
    name: 'clock_sync', cat: '__metadata', ph: 'M', ts: 777, pid: 2, tid: 3,
    args: { sync_id: 'grenadiers-capture-profile', secret },
  })
  const summary = sanitizeTraceEventsForTest(values)
  expect(summary.events).toHaveLength(512)
  expect(summary.truncated).toBe(true)
  expect(summary.droppedEvents).toBe(1)
  expect(summary.events[0]).toEqual({
    name: 'Display::DrawAndSwap', timestamp: 8, duration: 4, processId: 10, threadId: 11,
    category: 'viz', phase: 'X', flowId: 8, flowId2: 1_008,
  })
  expect(summary.traceClockTimestamp).toBe(777)
  expect(summary.eventAggregates.find((value) => value.name === 'Display::DrawAndSwap'))
    .toMatchObject({
      count: 520, totalDuration: 3_075, firstTimestamp: 0, lastTimestamp: 600,
      maxDuration: 999, unsupportedFlowIdEvents: 1, samplesTruncated: true,
    })
  expect(summary.eventAggregates.find((value) => value.name === 'Display::DrawAndSwap')
    ?.samples.some((event) => event.unsupportedFlowIds)).toBe(false)
  expect(JSON.stringify(summary)).not.toContain(secret)
  expect(JSON.stringify(summary)).not.toContain('network')
}

async function exerciseUnsupportedCategories(): Promise<void> {
  const commands: string[] = []
  const session = {
    send: async (method: string) => {
      commands.push(method)
      if (method === 'Tracing.getCategories') return { categories: ['cc', 'viz', 'gpu'] }
      throw new Error('unexpected protocol command')
    },
    detach: async () => undefined,
  }
  const page = {
    context: () => ({
      browser: () => ({
        version: () => '153.0.8010.12', newBrowserCDPSession: async () => session,
      }),
    }),
  } as unknown as Page
  const trace = new CaptureBrowserTrace()
  await trace.start(page)
  await trace.stop()
  expect(trace.summary.available).toBe(false)
  expect(trace.summary.unavailableReason).toBe('categories')
  expect(trace.summary.unavailableCategories).toEqual(['devtools'])
  expect(commands).toEqual(['Tracing.getCategories'])
}

async function exerciseTraceCleanupAndSanitization(): Promise<void> {
  const commands: string[] = []
  const listeners = new Map<string, (payload: never) => void>()
  let detached = false
  const session = {
    on: (event: string, listener: (payload: never) => void) => { listeners.set(event, listener) },
    once: (event: string, listener: (payload: never) => void) => { listeners.set(event, listener) },
    removeListener: (event: string, listener: (payload: never) => void) => {
      if (listeners.get(event) === listener) listeners.delete(event)
    },
    send: async (method: string) => {
      commands.push(method)
      if (method === 'Tracing.getCategories')
        return { categories: ['cc', 'viz', 'gpu', 'devtools'] }
      if (method === 'SystemInfo.getProcessInfo') return {
        processInfo: [
          { type: 'renderer', id: 12, cpuTime: 3 },
          { type: 'fixed-secret-process', id: 13, cpuTime: 4 },
        ],
      }
      if (method === 'Tracing.end') {
        listeners.get('Tracing.dataCollected')?.({ value: [{
          name: 'EncodeBitmapAsPngFast', cat: 'devtools', ph: 'X', ts: 5, pid: 12, tid: 8,
          args: { secret: 'fixed-trace-secret' },
        }] } as never)
        listeners.get('Tracing.tracingComplete')?.({ dataLossOccurred: false } as never)
      }
      return {}
    },
    detach: async () => { detached = true },
  }
  const page = {
    context: () => ({
      browser: () => ({
        version: () => '153.0.8010.12', newBrowserCDPSession: async () => session,
      }),
    }),
  } as unknown as Page
  const trace = new CaptureBrowserTrace()
  await trace.start(page)
  await trace.stop()
  expect(commands).toEqual([
    'Tracing.getCategories', 'SystemInfo.getProcessInfo', 'Tracing.start',
    'Tracing.recordClockSyncMarker', 'Tracing.end', 'SystemInfo.getProcessInfo',
  ])
  expect(detached).toBe(true)
  expect(listeners.size).toBe(0)
  expect(trace.summary.traceCompleted).toBe(true)
  expect(trace.summary.events).toEqual([{
    name: 'EncodeBitmapAsPngFast', timestamp: 5, processId: 12, threadId: 8,
    category: 'devtools', phase: 'X',
  }])
  expect(trace.summary.cpuAfter).toEqual([{ type: 'renderer', processId: 12, cpuTime: 3 }])
  expect(trace.summary).toMatchObject({
    cpuBeforeStatus: 'success', traceStartStatus: 'success', clockMarkerStatus: 'success',
    traceEndStatus: 'completed', cpuAfterStatus: 'success', detachStatus: 'success',
  })
  expect(JSON.stringify(trace.summary)).not.toContain('fixed-secret')
}

async function exerciseTraceFailureStatuses(): Promise<void> {
  const session = {
    on: () => undefined,
    once: () => undefined,
    removeListener: () => undefined,
    send: async (method: string) => {
      if (method === 'Tracing.getCategories')
        return { categories: ['cc', 'viz', 'gpu', 'devtools'] }
      if (method === 'SystemInfo.getProcessInfo') throw new Error('fixed-cpu-error')
      if (method === 'Tracing.recordClockSyncMarker') throw new Error('fixed-marker-error')
      if (method === 'Tracing.end') throw new Error('fixed-end-error')
      return {}
    },
    detach: async () => { throw new Error('fixed-detach-error') },
  }
  const page = {
    context: () => ({ browser: () => ({
      version: () => '153.0.8010.12', newBrowserCDPSession: async () => session,
    }) }),
  } as unknown as Page
  const trace = new CaptureBrowserTrace()
  await trace.start(page)
  await trace.stop()
  expect(trace.summary).toMatchObject({
    available: true, cpuBeforeStatus: 'error', traceStartStatus: 'success',
    clockMarkerStatus: 'error', traceEndStatus: 'command-error',
    cpuAfterStatus: 'error', detachStatus: 'error', traceCompleted: false,
  })
}

async function exerciseInactiveProfile(): Promise<void> {
  let commands = 0
  const page = new Proxy({}, {
    get() { commands += 1; throw new Error('inactive profile touched page') },
  }) as Page
  const profile = new CaptureWorkloadProfile(false)
  await profile.install([page])
  await profile.begin(page, {} as never)
  await profile.end('success')
  await profile.cleanup()
  await profile.attach({} as TestInfo)
  expect(commands).toBe(0)
  expect(profile.summary).toMatchObject({ enabled: false, canvases: [], renderCounters: [] })
}

async function exerciseProfileCleanupPriority(): Promise<void> {
  let traceStops = 0
  const page = {
    addInitScript: async () => undefined,
    evaluate: async (fn: { name?: string }) => {
      if (fn.name === 'stopAndReadRenderCounters') throw new Error('fixed-counter-read-error')
      return undefined
    },
    locator: () => ({ evaluate: async () => ({
      backingWidth: 1, backingHeight: 1, cssWidth: 1, cssHeight: 1,
      devicePixelRatio: 1, visibility: 'visible',
    }) }),
  } as unknown as Page
  const profile = new CaptureWorkloadProfile(true)
  const trace = {
    summary: profile.summary.browserTrace,
    start: async () => undefined,
    stop: async () => { traceStops += 1; throw new Error('fixed-trace-stop-error') },
  }
  ;(profile as unknown as { trace: typeof trace }).trace = trace
  await profile.install([page])
  await profile.begin(page, { state: {
    arena: { terrain: [], blocks: [] }, actors: [], bombs: [], flames: [], pickups: [],
  } } as never)
  await expect(profile.end('error')).resolves.toBeUndefined()
  await expect(profile.cleanup()).resolves.toBeUndefined()
  await expect(profile.attach({ attach: async () => {
    throw new Error('fixed-attachment-error')
  } } as unknown as TestInfo)).resolves.toBeUndefined()
  expect(traceStops).toBe(1)
  expect(profile.summary.failures).toEqual(['profile-stop', 'profile-stop'])
}

async function exercisePageCounters(page: Page): Promise<void> {
  await page.setContent('<main>safe render counter fixture</main>')
  await page.evaluate(() => {
    const calls: unknown[][] = []
    const result = {}
    const promiseResult = Promise.resolve(result)
    const failure = {}
    class FakeGl {
      drawArrays(...args: unknown[]) {
        if (this.constructor !== FakeGl) throw new Error('receiver changed')
        calls.push(['drawArrays', ...args])
        return result
      }
      drawElements(...args: unknown[]) { calls.push(['drawElements', ...args]); return result }
      drawArraysInstanced(...args: unknown[]) { calls.push(['drawArraysInstanced', ...args]); return result }
      drawElementsInstanced(...args: unknown[]) { calls.push(['drawElementsInstanced', ...args]); return result }
      bufferData(...args: unknown[]) { calls.push(['bufferData', ...args]); return promiseResult }
      bufferSubData(...args: unknown[]) { calls.push(['bufferSubData', ...args]); return result }
      texImage2D(...args: unknown[]) { calls.push(['texImage2D', ...args]); return result }
      texSubImage2D(value: unknown): never { calls.push(['texSubImage2D', value]); throw failure }
    }
    const target = window as typeof window & {
      WebGLRenderingContext: typeof FakeGl
      WebGL2RenderingContext: typeof FakeGl
      counterFixture: {
        calls: unknown[][]; result: object; promiseResult: Promise<object>
        failure: object; FakeGl: typeof FakeGl
      }
    }
    Object.defineProperty(target, 'WebGLRenderingContext', { configurable: true, value: FakeGl })
    Object.defineProperty(target, 'WebGL2RenderingContext', { configurable: true, value: FakeGl })
    target.counterFixture = { calls, result, promiseResult, failure, FakeGl }
  })
  await page.evaluate(installRenderCounters)
  await page.evaluate(installRenderCounters)
  await page.evaluate(resetRenderCounters)
  const transparency = await page.evaluate(() => {
    const fixture = (window as typeof window & {
      counterFixture: {
        calls: unknown[][]; result: object; promiseResult: Promise<object>; failure: object
        FakeGl: new () => {
        drawArrays: (...args: unknown[]) => object
        drawElements: (...args: unknown[]) => object
        drawArraysInstanced: (...args: unknown[]) => object
        drawElementsInstanced: (...args: unknown[]) => object
        bufferData: (...args: unknown[]) => Promise<object>
        bufferSubData: (...args: unknown[]) => object
        texImage2D: (...args: unknown[]) => object
        texSubImage2D: (value: unknown) => never
        }
      }
    }).counterFixture
    const gl = new fixture.FakeGl()
    let sameError = false
    try { gl.texSubImage2D(fixture.failure) } catch (error) { sameError = error === fixture.failure }
    return {
      sameResult: gl.drawArrays('mode', 2, 3) === fixture.result,
      invalidResult: gl.drawElements('mode', -1, 'type', 0) === fixture.result,
      instancedResult: gl.drawArraysInstanced('mode', 0, 4, 2) === fixture.result
        && gl.drawElementsInstanced('mode', 5, 'type', 0, 3) === fixture.result,
      bufferResult: gl.bufferData('target', new Uint8Array(7), 'usage') === fixture.promiseResult
        && gl.bufferSubData('target', 0, 11) === fixture.result,
      textureResult: gl.texImage2D('target', 0, 0, 0, 0, new Uint8Array(9)) === fixture.result,
      sameError, calls: fixture.calls.length,
    }
  })
  expect(transparency).toEqual({
    sameResult: true, invalidResult: true, instancedResult: true,
    bufferResult: true, textureResult: true, sameError: true, calls: 8,
  })
  const counters = await page.evaluate(stopAndReadRenderCounters)
  expect(counters.buckets).toHaveLength(1)
  expect(counters.buckets[0]).toMatchObject({
    drawCalls: 4, submittedVertices: 11, submittedIndices: 15,
    submittedInstances: 7, bufferUploadCalls: 2, bufferUploadSourceSpanBytesUpperBound: 7,
    textureUploadCalls: 2, textureUploadSourceSpanBytesUpperBound: 9,
    unknownBufferUploadSourceSpan: true, unknownTextureUploadSourceSpan: true,
  })
  await page.evaluate(() => {
    const fixture = (window as typeof window & {
      counterFixture: { FakeGl: new () => { drawArrays: (...args: unknown[]) => object } }
    }).counterFixture
    new fixture.FakeGl().drawArrays('after-stop', 0, 99)
  })
  expect(await page.evaluate(readRenderCounters)).toEqual(counters)
  await page.evaluate(removeRenderCounters)
  expect(await page.evaluate(readRenderCounters)).toEqual({ buckets: [], truncated: false })
}

export async function exerciseCaptureProfileFixtures(page: Page): Promise<void> {
  await exerciseTraceSanitizer()
  await exerciseUnsupportedCategories()
  await exerciseTraceCleanupAndSanitization()
  await exerciseTraceFailureStatuses()
  await exerciseInactiveProfile()
  await exerciseProfileCleanupPriority()
  await exercisePageCounters(page)
}
