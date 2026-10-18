import type { Page } from '@playwright/test'

type RenderCounterBucket = {
  second: number
  animationCallbacks: number
  drawCalls: number
  submittedVertices: number
  submittedIndices: number
  submittedInstances: number
  bufferUploadCalls: number
  bufferUploadSourceSpanBytesUpperBound: number
  textureUploadCalls: number
  textureUploadSourceSpanBytesUpperBound: number
  unknownBufferUploadSourceSpan: boolean
  unknownTextureUploadSourceSpan: boolean
}

export type PageRenderCounters = { buckets: RenderCounterBucket[]; truncated: boolean }

type CounterWindow = Window & {
  __grenadiersRenderCounters?: {
    buckets: RenderCounterBucket[]
    truncated: boolean
    startedAt: number
    active: boolean
    originals: Array<{ target: object; name: string; descriptor?: PropertyDescriptor }>
    wrapped: WeakSet<object>
  }
}

export function installRenderCounters(): void {
  const target = window as CounterWindow
  if (target.__grenadiersRenderCounters) return
  const state: NonNullable<CounterWindow['__grenadiersRenderCounters']> = {
    buckets: [], truncated: false, startedAt: performance.now(), active: true, originals: [],
    wrapped: new WeakSet<object>(),
  }
  target.__grenadiersRenderCounters = state
  const bucket = (): RenderCounterBucket => {
    const second = Math.max(0, Math.floor((performance.now() - state.startedAt) / 1_000))
    let current = state.buckets.at(-1)
    if (!current || current.second !== second) {
      current = {
        second, animationCallbacks: 0, drawCalls: 0, submittedVertices: 0,
        submittedIndices: 0, submittedInstances: 0, bufferUploadCalls: 0,
        bufferUploadSourceSpanBytesUpperBound: 0, textureUploadCalls: 0,
        textureUploadSourceSpanBytesUpperBound: 0, unknownBufferUploadSourceSpan: false,
        unknownTextureUploadSourceSpan: false,
      }
      state.buckets.push(current)
      if (state.buckets.length > 128) {
        state.buckets.shift()
        state.truncated = true
      }
    }
    return current
  }
  const bytes = (values: unknown[]): number | undefined => {
    for (let index = values.length - 1; index >= 0; index -= 1) {
      const value = values[index]
      if (value instanceof ArrayBuffer) return value.byteLength
      if (ArrayBuffer.isView(value)) return value.byteLength
    }
    return undefined
  }
  const replace = (object: Record<string, unknown>, name: string,
    measure: (current: RenderCounterBucket, args: unknown[]) => void): void => {
    const original = object[name]
    if (typeof original !== 'function') return
    if (state.wrapped.has(original)) return
    state.wrapped.add(original)
    const descriptor = Object.getOwnPropertyDescriptor(object, name)
    if (descriptor && !descriptor.configurable) return
    state.originals.push({ target: object, name, descriptor })
    const wrapped = function (this: unknown, ...args: unknown[]): unknown {
      if (state.active) measure(bucket(), args)
      return Reflect.apply(original, this, args)
    }
    state.wrapped.add(wrapped)
    Object.defineProperty(object, name, {
      configurable: true, enumerable: descriptor?.enumerable ?? false, writable: true,
      value: wrapped,
    })
  }
  const integer = (value: unknown): number => typeof value === 'number'
    && Number.isSafeInteger(value) && value > 0 ? value : 0
  const wrapGl = (prototype: Record<string, unknown>): void => {
    if (state.wrapped.has(prototype)) return
    state.wrapped.add(prototype)
    replace(prototype, 'drawArrays', (current, args) => {
      current.drawCalls += 1
      current.submittedVertices += integer(args[2])
      current.submittedInstances += 1
    })
    replace(prototype, 'drawElements', (current, args) => {
      current.drawCalls += 1
      current.submittedIndices += integer(args[1])
      current.submittedInstances += 1
    })
    replace(prototype, 'drawArraysInstanced', (current, args) => {
      const instances = integer(args[3])
      current.drawCalls += 1
      current.submittedVertices += integer(args[2]) * instances
      current.submittedInstances += instances
    })
    replace(prototype, 'drawElementsInstanced', (current, args) => {
      const instances = integer(args[4])
      current.drawCalls += 1
      current.submittedIndices += integer(args[1]) * instances
      current.submittedInstances += instances
    })
    for (const name of ['bufferData', 'bufferSubData']) replace(prototype, name, (current, args) => {
      current.bufferUploadCalls += 1
      const size = bytes(args)
      if (size === undefined) current.unknownBufferUploadSourceSpan = true
      else current.bufferUploadSourceSpanBytesUpperBound += size
    })
    for (const name of [
      'texImage2D', 'texSubImage2D', 'compressedTexImage2D', 'compressedTexSubImage2D',
      'texImage3D', 'texSubImage3D', 'compressedTexImage3D', 'compressedTexSubImage3D',
    ]) replace(prototype, name, (current, args) => {
      current.textureUploadCalls += 1
      const size = bytes(args)
      if (size === undefined) current.unknownTextureUploadSourceSpan = true
      else current.textureUploadSourceSpanBytesUpperBound += size
    })
  }
  const gl = (target as unknown as { WebGLRenderingContext?: { prototype: object } })
    .WebGLRenderingContext?.prototype
  const gl2 = (target as unknown as { WebGL2RenderingContext?: { prototype: object } })
    .WebGL2RenderingContext?.prototype
  if (gl) wrapGl(gl as Record<string, unknown>)
  if (gl2) wrapGl(gl2 as Record<string, unknown>)
  replace(target as unknown as Record<string, unknown>, 'requestAnimationFrame', (_current, args) => {
    const callback = args[0]
    if (typeof callback !== 'function') return
    args[0] = function (this: unknown, ...callbackArgs: unknown[]): unknown {
      if (state.active) bucket().animationCallbacks += 1
      return Reflect.apply(callback, this, callbackArgs)
    }
  })
}

export function resetRenderCounters(): void {
  const state = (window as CounterWindow).__grenadiersRenderCounters
  if (!state) return
  state.buckets.length = 0
  state.truncated = false
  state.startedAt = performance.now()
  state.active = true
}

export function readRenderCounters(): PageRenderCounters {
  const state = (window as CounterWindow).__grenadiersRenderCounters
  return state ? { buckets: state.buckets.map((bucket) => ({ ...bucket })), truncated: state.truncated }
    : { buckets: [], truncated: false }
}

export function stopAndReadRenderCounters(): PageRenderCounters {
  const state = (window as CounterWindow).__grenadiersRenderCounters
  if (state) state.active = false
  return state ? { buckets: state.buckets.map((bucket) => ({ ...bucket })), truncated: state.truncated }
    : { buckets: [], truncated: false }
}

export function removeRenderCounters(): void {
  const target = window as CounterWindow
  const state = target.__grenadiersRenderCounters
  if (!state) return
  for (const original of state.originals.reverse()) {
    if (original.descriptor) Object.defineProperty(original.target, original.name, original.descriptor)
    else Reflect.deleteProperty(original.target, original.name)
  }
  delete target.__grenadiersRenderCounters
}

export async function preparePageRenderCounters(page: Page): Promise<void> {
  await page.addInitScript(installRenderCounters)
  await page.evaluate(installRenderCounters)
}
