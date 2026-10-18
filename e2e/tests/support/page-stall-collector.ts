export type CollectorEventLabel = 'initialized' | 'visible' | 'hidden' | 'focused' | 'blurred'
  | 'longTask' | 'schedulingGap' | 'bindingDuration' | 'glCall'

export type PageCollectorEvent = {
  label: CollectorEventLabel
  pageAt: number
  documentOrdinal: number
  timeOrigin: number
  duration?: number
  method?: string
  contextOrdinal?: number
  callCount?: number
  stack?: Array<{ functionName: string; asset: string; line: number; column: number }>
}

type DiagnosticBinding = (event: PageCollectorEvent) => Promise<void>
type DiagnosticWindow = Window & {
  grenadiersStallDiagnostic?: DiagnosticBinding
  grenadiersStallCollectorInstalled?: true
  WebGLRenderingContext?: { prototype: object }
  WebGL2RenderingContext?: { prototype: object }
}

export function installPageCollector(input: {
  intervalMs: number; reportingGapMs: number; slowGlCallMs: number
}): void {
  const observed = window as DiagnosticWindow
  const binding = observed.grenadiersStallDiagnostic
  if (!binding || observed.grenadiersStallCollectorInstalled) return
  observed.grenadiersStallCollectorInstalled = true
  let documentOrdinal = 0
  let observedRoot: Element | null = null
  const send = (event: Omit<PageCollectorEvent, 'documentOrdinal' | 'timeOrigin'>): void => {
    const started = performance.now()
    const contextual = { ...event, documentOrdinal, timeOrigin: performance.timeOrigin }
    void binding(contextual).then(() => binding({
      label: 'bindingDuration', pageAt: performance.now(), documentOrdinal,
      timeOrigin: performance.timeOrigin, duration: performance.now() - started,
    })).catch(() => undefined)
  }
  const state = (): void => send({
    label: document.visibilityState === 'visible' ? 'visible' : 'hidden',
    pageAt: performance.now(),
  })
  const initializeDocument = (): void => {
    documentOrdinal += 1
    observedRoot = document.documentElement
    send({ label: 'initialized', pageAt: performance.now() })
    state()
    document.addEventListener('visibilitychange', state)
  }
  initializeDocument()
  send({ label: document.hasFocus() ? 'focused' : 'blurred', pageAt: performance.now() })
  addEventListener('focus', () => send({ label: 'focused', pageAt: performance.now() }))
  addEventListener('blur', () => send({ label: 'blurred', pageAt: performance.now() }))
  if ('PerformanceObserver' in window) {
    try {
      new PerformanceObserver((list) => {
        for (const entry of list.getEntries()) {
          send({ label: 'longTask', pageAt: entry.startTime, duration: entry.duration })
        }
      }).observe({ type: 'longtask', buffered: true })
    } catch { /* Long-task observation is optional in this browser. */ }
  }

  const methods = ['useProgram', 'deleteBuffer', 'deleteProgram', 'deleteShader', 'deleteTexture', 'deleteFramebuffer',
    'deleteRenderbuffer', 'deleteVertexArray', 'drawArrays', 'drawElements', 'drawArraysInstanced', 'drawElementsInstanced',
    'clear', 'getExtension', 'deleteQuery', 'deleteSampler', 'deleteSync', 'deleteTransformFeedback', 'drawRangeElements'] as const
  const contextOrdinals = new WeakMap<object, number>(), callCounts = new WeakMap<object, Map<string, number>>()
  const wrappedExtensions = new WeakSet<object>(); let nextContextOrdinal = 1
  const ordinalFor = (context: object): number => {
    const retained = contextOrdinals.get(context)
    if (retained !== undefined) return retained
    const ordinal = nextContextOrdinal++
    contextOrdinals.set(context, ordinal)
    return ordinal
  }
  const stackFrames = (): PageCollectorEvent['stack'] => {
    const frames: NonNullable<PageCollectorEvent['stack']> = []
    for (const raw of (new Error().stack ?? '').split('\n').slice(1, 9)) {
      const line = raw.slice(0, 2_048).trim()
      const position = line.match(/:(\d+):(\d+)\)?$/)
      if (!line.startsWith('at ') || !position || position.index === undefined) continue
      const prefix = line.slice(3, position.index)
      const open = prefix.lastIndexOf(' (')
      const candidate = open < 0 ? '' : prefix.slice(0, open).trim()
      const functionName = /^[A-Za-z_$][\w$]*(?:\.[A-Za-z_$][\w$]*)*$/.test(candidate)
        ? candidate.slice(0, 64) : 'anonymous'
      const location = (open < 0 ? prefix : prefix.slice(open + 2)).split(/[?#]/, 1)[0]
      const basename = location.split('/').at(-1) ?? 'anonymous'
      const asset = /^[A-Za-z0-9._-]+$/.test(basename) ? basename.slice(0, 64) : 'anonymous'
      frames.push({
        functionName, asset,
        line: Math.max(0, Number.parseInt(position[1], 10)),
        column: Math.max(0, Number.parseInt(position[2], 10)),
      })
    }
    return frames
  }
  const measure = (call: { context: object; method: string; native: (...args: unknown[]) => unknown
    receiver: unknown; args: unknown[]; after?: (result: unknown) => void }): unknown => {
    const { context, method, native, receiver, args, after } = call
    let ordinal = 0, count = 0, started = 0
    let timing = false
    try {
      ordinal = ordinalFor(context)
      let contextCounts = callCounts.get(context)
      if (!contextCounts) { contextCounts = new Map(); callCounts.set(context, contextCounts) }
      count = (contextCounts.get(method) ?? 0) + 1
      contextCounts.set(method, count)
      started = performance.now()
      timing = true
    } catch { /* Native invocation remains authoritative. */ }
    let result: unknown, completed = false
    try {
      result = Reflect.apply(native, receiver, args)
      completed = true
      return result
    } finally {
      try {
        if (timing) {
          const finished = performance.now()
          const duration = Math.max(0, finished - started)
          if (duration >= input.slowGlCallMs) {
            send({
              label: 'glCall', method, contextOrdinal: ordinal, callCount: count,
              pageAt: finished, duration, stack: stackFrames(),
            })
          }
        }
      } catch { /* Diagnostics cannot replace a native return or exception. */ }
      try { if (completed) after?.(result) } catch { /* Preserve the native return. */ }
    }
  }
  const wrapExtension = (extension: object, context: object): void => {
    if (wrappedExtensions.has(extension)) return
    try {
      const native = Reflect.get(extension, 'loseContext') as unknown
      if (typeof native !== 'function') return
      const own = Object.getOwnPropertyDescriptor(extension, 'loseContext')
      Object.defineProperty(extension, 'loseContext', {
        configurable: own?.configurable ?? true,
        enumerable: own?.enumerable ?? false,
        writable: own?.writable ?? true,
        value: function(this: unknown, ...args: unknown[]) {
          return measure({ context, method: 'loseContext',
            native: native as (...values: unknown[]) => unknown, receiver: this, args })
        },
      })
      wrappedExtensions.add(extension)
    } catch { /* Some browser extension objects may reject an own wrapper. */ }
  }
  const wrapPrototype = (prototype: object): void => {
    for (const method of methods) {
      try {
        const descriptor = Object.getOwnPropertyDescriptor(prototype, method)
        if (!descriptor || typeof descriptor.value !== 'function') continue
        const native = descriptor.value as (...args: unknown[]) => unknown
        Object.defineProperty(prototype, method, {
          ...descriptor,
          value: function(this: object, ...args: unknown[]) {
            const after = method === 'getExtension' && args[0] === 'WEBGL_lose_context'
              ? (result: unknown): void => {
                if (typeof result === 'object' && result !== null) wrapExtension(result, this)
              } : undefined
            return measure({ context: this, method, native, receiver: this, args, after })
          },
        })
      } catch { /* An unavailable or immutable native method remains untouched. */ }
    }
  }
  if (observed.WebGLRenderingContext) wrapPrototype(observed.WebGLRenderingContext.prototype)
  if (observed.WebGL2RenderingContext) wrapPrototype(observed.WebGL2RenderingContext.prototype)

  let expected = performance.now() + input.intervalMs
  setInterval(() => {
    const now = performance.now()
    if (observedRoot !== document.documentElement) initializeDocument()
    const gap = Math.max(0, now - expected)
    if (gap >= input.reportingGapMs) {
      send({ label: 'schedulingGap', pageAt: now, duration: gap })
    }
    expected = now + input.intervalMs
  }, input.intervalMs)
}
