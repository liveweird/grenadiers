import type { ChildProcess } from 'node:child_process'

const HISTORY_LIMIT = 16_384
const CARRY_LIMIT = 4_096
const ENDPOINT = /DevTools listening on (ws:\/\/[^\s]+)/
const CDP_CAPABILITY = /ws:\/\/[^\s]+/g

type StartupPhase = 'waiting' | 'connected' | 'timeout' | 'spawn-error' | 'early-exit'

export type VisibilityStartupDiagnostics = {
  phase: StartupPhase
  elapsedMs: number
  endpointSeen: boolean
  stderrBytes: number
  stderrTail: string
  stderrTruncated: boolean
  incompleteLineBytes: number
  omittedIncompleteLineBytes?: number
  spawnError?: { name: string; code?: string; message: string }
  exit?: { code: number | null; signal: NodeJS.Signals | null }
}

type StartupChild = Pick<ChildProcess, 'stderr' | 'once' | 'removeListener'>
type TimerScheduler = {
  set: (callback: () => void, timeout: number) => unknown
  clear: (timer: unknown) => void
}

export type EndpointWait = {
  endpoint: Promise<string>
  diagnostics: () => VisibilityStartupDiagnostics
  dispose: () => void
}

export function sanitizeStartupText(text: string, profile: string): string {
  return text.replaceAll(profile, '<redacted-profile>')
    .replace(CDP_CAPABILITY, '<redacted-cdp-endpoint>')
}

function errorCode(error: Error): string | undefined {
  const code = (error as NodeJS.ErrnoException).code
  return typeof code === 'string' ? code : undefined
}

export function sanitizePublicError(error: unknown, profile: string,
  seen = new WeakMap<object, Error>()): Error {
  if (!(error instanceof Error)) return new Error('Non-Error startup failure')
  const known = seen.get(error)
  if (known) return known
  const message = sanitizeStartupText(error.message, profile)
  const clone = error instanceof AggregateError
    ? new AggregateError([], message)
    : new Error(message)
  seen.set(error, clone)
  clone.name = error.name
  if (error.stack) clone.stack = sanitizeStartupText(error.stack, profile)
  const code = errorCode(error)
  if (code) Object.defineProperty(clone, 'code', { value: code, enumerable: true })
  if (error instanceof AggregateError) Object.defineProperty(clone, 'errors', {
    value: error.errors.map((nested) => sanitizePublicError(nested, profile, seen)),
    enumerable: false,
  })
  if (error.cause !== undefined) Object.defineProperty(clone, 'cause', {
    value: sanitizePublicError(error.cause, profile, seen), enumerable: false,
  })
  return clone
}

export function publicErrorSummary(error: unknown): string {
  return error instanceof Error ? `${error.name}: ${error.message}` : 'Non-Error failure'
}

export async function runSanitizedCleanup(cleanup: () => Promise<void>, profile: string): Promise<void> {
  try {
    await cleanup()
  } catch (error) {
    throw sanitizePublicError(error, profile)
  }
}

function sanitizedError(error: Error, profile: string): { name: string; code?: string; message: string } {
  const code = errorCode(error)
  return { name: error.name, ...(code ? { code } : {}),
    message: sanitizeStartupText(error.message, profile) }
}

function utf8Tail(text: string, limit: number): string {
  const encoded = Buffer.from(text)
  if (encoded.length <= limit) return text
  let start = encoded.length - limit
  while (start < encoded.length && (encoded[start] & 0xc0) === 0x80) start += 1
  return encoded.subarray(start).toString('utf8')
}

export class ChromiumStartupStream {
  private readonly profile: string
  private carry = Buffer.alloc(0)
  private omittedIncomplete = 0
  private history = ''
  private bytes = 0
  private truncated = false
  private seen = false

  constructor(profile: string) { this.profile = profile }

  push(chunk: Buffer | string): string | undefined {
    const input = Buffer.isBuffer(chunk) ? chunk : Buffer.from(chunk)
    this.bytes += input.length
    let start = 0
    let endpoint: string | undefined
    for (let index = 0; index < input.length; index += 1) {
      if (input[index] !== 0x0a) continue
      const found = this.completeLine(input.subarray(start, index))
      endpoint ??= found
      start = index + 1
    }
    this.retainIncomplete(input.subarray(start))
    return endpoint
  }

  private completeLine(part: Buffer): string | undefined {
    if (this.omittedIncomplete > 0) {
      const omitted = this.omittedIncomplete + part.length
      this.omittedIncomplete = 0
      this.carry = Buffer.alloc(0)
      this.appendHistory(`[omitted oversized stderr line: ${omitted} bytes]\n`)
      return undefined
    }
    const line = Buffer.concat([this.carry, part])
    this.carry = Buffer.alloc(0)
    if (line.length > CARRY_LIMIT) {
      this.truncated = true
      this.appendHistory(`[omitted oversized stderr line: ${line.length} bytes]\n`)
      return undefined
    }
    const withoutCr = line.at(-1) === 0x0d ? line.subarray(0, -1) : line
    const text = withoutCr.toString('utf8')
    this.appendHistory(`${sanitizeStartupText(text, this.profile)}\n`)
    return this.endpoint(text)
  }

  private retainIncomplete(part: Buffer): void {
    if (part.length === 0) return
    if (this.omittedIncomplete > 0) {
      this.omittedIncomplete += part.length
      return
    }
    if (this.carry.length + part.length <= CARRY_LIMIT) {
      this.carry = Buffer.concat([this.carry, part])
      return
    }
    this.omittedIncomplete = this.carry.length + part.length
    this.carry = Buffer.alloc(0)
    this.truncated = true
  }

  private appendHistory(text: string): void {
    this.history += text
    if (Buffer.byteLength(this.history) <= HISTORY_LIMIT) return
    this.history = utf8Tail(this.history, HISTORY_LIMIT)
    this.truncated = true
  }

  private endpoint(line: string): string | undefined {
    const match = ENDPOINT.exec(line)
    if (!match) return undefined
    try {
      const parsed = new URL(match[1])
      if (parsed.protocol !== 'ws:' || parsed.href !== match[1]) return undefined
      this.seen = true
      return match[1]
    } catch {
      return undefined
    }
  }

  diagnostics(): Pick<VisibilityStartupDiagnostics, 'endpointSeen' | 'stderrBytes' | 'stderrTail'
    | 'stderrTruncated' | 'incompleteLineBytes' | 'omittedIncompleteLineBytes'> {
    let incompleteMarker = ''
    if (this.omittedIncomplete > 0) {
      incompleteMarker = `[omitted incomplete stderr line: ${this.omittedIncomplete} bytes]\n`
    } else if (this.carry.length > 0) {
      incompleteMarker = `[incomplete stderr line: ${this.carry.length} bytes]\n`
    }
    return {
      endpointSeen: this.seen,
      stderrBytes: this.bytes,
      stderrTail: utf8Tail(this.history + incompleteMarker, HISTORY_LIMIT),
      stderrTruncated: this.truncated,
      incompleteLineBytes: this.carry.length,
      ...(this.omittedIncomplete > 0
        ? { omittedIncompleteLineBytes: this.omittedIncomplete } : {}),
    }
  }
}

export function waitForChromiumEndpoint(
  child: StartupChild,
  profile: string,
  timeout = 10_000,
  now: () => number = () => performance.now(),
  scheduler: TimerScheduler = {
    set: (callback, delay) => setTimeout(callback, delay),
    clear: (timer) => clearTimeout(timer as NodeJS.Timeout),
  },
): EndpointWait {
  const started = now()
  const stream = new ChromiumStartupStream(profile)
  let phase: StartupPhase = 'waiting'
  let spawnError: VisibilityStartupDiagnostics['spawnError']
  let exit: VisibilityStartupDiagnostics['exit']
  let settled = false
  let resolveEndpoint!: (endpoint: string) => void
  let rejectEndpoint!: (error: Error) => void
  const timer: { value?: unknown } = {}
  const endpoint = new Promise<string>((resolve, reject) => {
    resolveEndpoint = resolve
    rejectEndpoint = reject
  })
  const settle = (next: Exclude<StartupPhase, 'waiting'>, outcome: string | Error): void => {
    if (settled) return
    settled = true
    phase = next
    scheduler.clear(timer.value)
    child.removeListener('error', onError)
    child.removeListener('exit', onExit)
    if (typeof outcome === 'string') resolveEndpoint(outcome)
    else rejectEndpoint(outcome)
  }
  const onData = (chunk: Buffer | string): void => {
    const found = stream.push(chunk)
    if (found) settle('connected', found)
  }
  const onError = (error: Error): void => {
    spawnError = sanitizedError(error, profile)
    settle('spawn-error', new Error(`Chromium spawn failed: ${spawnError.message}`, {
      cause: sanitizePublicError(error, profile),
    }))
  }
  const onExit = (code: number | null, signal: NodeJS.Signals | null): void => {
    exit = { code, signal }
    settle('early-exit', new Error(`Chromium exited before connection (code=${code}, signal=${signal})`))
  }
  timer.value = scheduler.set(
    () => settle('timeout', new Error('Chromium debugging endpoint timed out')), timeout)
  child.once('error', onError)
  child.once('exit', onExit)
  child.stderr?.on('data', onData)
  return {
    endpoint,
    diagnostics: () => ({
      phase,
      elapsedMs: Math.max(0, now() - started),
      ...stream.diagnostics(),
      ...(spawnError ? { spawnError } : {}),
      ...(exit ? { exit } : {}),
    }),
    dispose() {
      scheduler.clear(timer.value)
      child.removeListener('error', onError)
      child.removeListener('exit', onExit)
      child.stderr?.removeListener('data', onData)
    },
  }
}

export function startupDiagnosticsFrom(error: unknown): VisibilityStartupDiagnostics | undefined {
  if (error && typeof error === 'object' && 'startupDiagnostics' in error) {
    return (error as { startupDiagnostics?: VisibilityStartupDiagnostics }).startupDiagnostics
  }
  if (error instanceof AggregateError) {
    for (const nested of error.errors) {
      const diagnostics = startupDiagnosticsFrom(nested)
      if (diagnostics) return diagnostics
    }
  }
  return undefined
}

export class VisibilityStartupError extends Error {
  readonly startupDiagnostics: VisibilityStartupDiagnostics

  constructor(message: string, startupDiagnostics: VisibilityStartupDiagnostics, options?: ErrorOptions) {
    super(message, options)
    this.name = 'VisibilityStartupError'
    this.startupDiagnostics = startupDiagnostics
  }
}
