import assert from 'node:assert/strict'
import { EventEmitter } from 'node:events'
import { PassThrough } from 'node:stream'
import {
  ChromiumStartupStream, publicErrorSummary, runSanitizedCleanup, sanitizePublicError,
  VisibilityStartupError, startupDiagnosticsFrom, waitForChromiumEndpoint,
} from './visibility-startup.ts'

type FakeChild = EventEmitter & { stderr: PassThrough }

function child(): FakeChild {
  const events = new EventEmitter() as FakeChild
  events.stderr = new PassThrough()
  return events
}

function scheduler() {
  let callback: (() => void) | undefined
  let cleared = 0
  return {
    timer: {
      set(next: () => void) { callback = next; return 1 },
      clear() { cleared += 1 },
    },
    fire() { assert(callback); callback() },
    cleared: () => cleared,
  }
}

async function expectRejected(promise: Promise<unknown>, pattern: RegExp): Promise<Error> {
  try {
    await promise
  } catch (error) {
    assert(error instanceof Error)
    assert.match(error.message, pattern)
    return error
  }
  throw new Error('expected rejection')
}

function serializedError(error: Error): string {
  const nested = error instanceof AggregateError ? error.errors : []
  return JSON.stringify({
    name: error.name, message: error.message, stack: error.stack,
    code: (error as NodeJS.ErrnoException).code,
    cause: error.cause instanceof Error ? JSON.parse(serializedError(error.cause)) : error.cause,
    errors: nested.map((item) => item instanceof Error
      ? JSON.parse(serializedError(item)) : item),
  })
}

{
  const fake = child()
  const wait = waitForChromiumEndpoint(fake as never, '/private/profile')
  const endpoint = 'ws://127.0.0.1:9222/devtools/browser/private-capability'
  fake.stderr.write(`noise\nDevTools listening on ${endpoint}\nimportant diagnostic\n${'x'.repeat(20_000)}`)
  assert.equal(await wait.endpoint, endpoint)
  const receipt = wait.diagnostics()
  assert.equal(receipt.phase, 'connected')
  assert.equal(receipt.endpointSeen, true)
  assert.equal(receipt.stderrTail.includes('private-capability'), false)
  assert.match(receipt.stderrTail, /important diagnostic/)
  assert(receipt.stderrTail.length <= 16_384)
  wait.dispose()
}

{
  const fake = child()
  const wait = waitForChromiumEndpoint(fake as never, '/profile')
  let settled = false
  void wait.endpoint.finally(() => { settled = true })
  fake.stderr.write('DevTools listening on ws://127.0.0.1:9222')
  await Promise.resolve()
  assert.equal(settled, false)
  fake.stderr.write('/devtools/browser/split')
  await Promise.resolve()
  assert.equal(settled, false)
  fake.stderr.write('\r\n')
  assert.equal(await wait.endpoint, 'ws://127.0.0.1:9222/devtools/browser/split')
  wait.dispose()
}

{
  const stream = new ChromiumStartupStream('/secret/profile')
  for (let index = 0; index < 200; index += 1) stream.push(`noise-${index}-${'z'.repeat(500)}\r\n`)
  stream.push(`unterminated-${'q'.repeat(10_000)}`)
  const receipt = stream.diagnostics()
  assert(receipt.stderrBytes > 100_000)
  assert(Buffer.byteLength(receipt.stderrTail) <= 16_384)
  assert(receipt.incompleteLineBytes <= 4_096)
  assert.equal(receipt.stderrTruncated, true)
}

{
  const profile = '/private/secret-profile-token'
  const stream = new ChromiumStartupStream(profile)
  stream.push(`${'safe\n'.repeat(4_000)}profile=${profile.slice(0, 12)}`)
  stream.push(`${profile.slice(12)} endpoint=ws://127.0.0.1:9222/devtools/browser/private-token\n`)
  const receipt = stream.diagnostics()
  assert.equal(receipt.stderrTail.includes('secret-profile-token'), false)
  assert.equal(receipt.stderrTail.includes('private-token'), false)
  assert.match(receipt.stderrTail, /<redacted-profile>/)
  assert.match(receipt.stderrTail, /<redacted-cdp-endpoint>/)
  assert(Buffer.byteLength(receipt.stderrTail) <= 16_384)
}

{
  const stream = new ChromiumStartupStream('/profile')
  const unicode = Buffer.from(`${'é'.repeat(12_000)}\n`)
  stream.push(unicode.subarray(0, 17_001))
  stream.push(unicode.subarray(17_001))
  stream.push(Buffer.from([0xe2, 0x82]))
  let receipt = stream.diagnostics()
  assert.equal(receipt.stderrBytes, unicode.length + 2)
  assert(Buffer.byteLength(receipt.stderrTail) <= 16_384)
  assert(receipt.incompleteLineBytes <= 4_096)
  assert.equal(receipt.stderrTail.includes('\ufffd'), false)
  stream.push(Buffer.from([0xac, 0x0a]))
  receipt = stream.diagnostics()
  assert.equal(receipt.stderrBytes, unicode.length + 4)
  assert.equal(receipt.stderrTail.includes('\ufffd'), false)
}

{
  let now = 10
  const fake = child()
  const scheduled = scheduler()
  const wait = waitForChromiumEndpoint(fake as never, '/profile', 10_000,
    () => now, scheduled.timer)
  fake.stderr.write('diagnostic /profile ws://127.0.0.1:1/private-without-newline')
  now = 10_010
  scheduled.fire()
  await expectRejected(wait.endpoint, /timed out/)
  const receipt = wait.diagnostics()
  assert.equal(receipt.phase, 'timeout')
  assert.equal(receipt.elapsedMs, 10_000)
  assert.equal(receipt.stderrTail.includes('/profile'), false)
  assert.equal(receipt.stderrTail.includes('private-without-newline'), false)
  wait.dispose()
}

{
  const fake = child()
  const wait = waitForChromiumEndpoint(fake as never, '/secret/profile')
  const error = Object.assign(new Error('cannot use /secret/profile ws://127.0.0.1:2/token'),
    { code: 'ENOENT' })
  fake.emit('error', error)
  await expectRejected(wait.endpoint, /spawn failed/)
  const receipt = wait.diagnostics()
  assert.equal(receipt.phase, 'spawn-error')
  assert.equal(receipt.spawnError?.code, 'ENOENT')
  assert.equal(receipt.spawnError?.message.includes('/secret/profile'), false)
  assert.equal(receipt.spawnError?.message.includes('/token'), false)
  wait.dispose()
}

{
  const profile = '/private/profile'
  const capability = 'ws://127.0.0.1:9222/devtools/browser/private-capability'
  const nested = new Error(`nested ${capability} ${profile}`)
  nested.stack = `Error: nested ${capability}\n at ${profile}/source.ts:1`
  const aggregate = new AggregateError([nested, new Error(`cleanup ${profile}`)],
    `primary ${capability}`, { cause: nested })
  const safe = sanitizePublicError(aggregate, profile)
  const serialized = serializedError(safe)
  assert.equal(serialized.includes('private-capability'), false)
  assert.equal(serialized.includes('/private/profile'), false)
  assert.match(serialized, /redacted-cdp-endpoint/)
  assert.match(serialized, /redacted-profile/)
  assert(safe instanceof AggregateError)
  assert.equal(safe.cause, safe.errors[0])
}

{
  const profile = '/private/secret-profile'
  const capability = 'ws://127.0.0.1:9222/devtools/browser/cleanup-capability'
  const cleanup = new Error(`rm failed for ${profile}`, {
    cause: new Error(`nested cleanup ${capability}`),
  })
  const rejected = await expectRejected(
    runSanitizedCleanup(async () => { throw cleanup }, profile), /rm failed/)
  const serialized = serializedError(rejected)
  assert.equal(serialized.includes(profile), false)
  assert.equal(serialized.includes('cleanup-capability'), false)
  assert.match(serialized, /redacted-profile/)
  assert.match(serialized, /redacted-cdp-endpoint/)
  const primary = sanitizePublicError(new Error(`startup ${capability}`), profile)
  const message = `primary (${publicErrorSummary(primary)}); cleanup (${publicErrorSummary(rejected)})`
  assert.match(message, /primary \(Error: startup <redacted-cdp-endpoint>\)/)
  assert.match(message, /cleanup \(Error: rm failed for <redacted-profile>\)/)
}

{
  const fake = child()
  const profile = '/private/profile'
  const wait = waitForChromiumEndpoint(fake as never, profile)
  const cause = new Error(`cause ws://127.0.0.1:9222/devtools/browser/private ${profile}`)
  fake.emit('error', new Error(`spawn ${profile}`, { cause }))
  const rejected = await expectRejected(wait.endpoint, /spawn failed/)
  const serialized = serializedError(rejected)
  assert.equal(serialized.includes('/private/profile'), false)
  assert.equal(serialized.includes('/private'), false)
  wait.dispose()
}

for (const [code, signal] of [[7, null], [null, 'SIGABRT']] as const) {
  const fake = child()
  const wait = waitForChromiumEndpoint(fake as never, '/profile')
  fake.emit('exit', code, signal)
  await expectRejected(wait.endpoint, /exited before connection/)
  assert.deepEqual(wait.diagnostics().exit, { code, signal })
  wait.dispose()
}

{
  const fake = child()
  const scheduled = scheduler()
  const wait = waitForChromiumEndpoint(fake as never, '/profile', 10_000,
    () => 1, scheduled.timer)
  fake.stderr.write('DevTools listening on ws://127.0.0.1:3/devtools/browser/success\n')
  const endpoint = await wait.endpoint
  fake.emit('exit', 9, 'SIGKILL')
  fake.stderr.write('late diagnostic\n')
  scheduled.fire()
  assert.equal(await wait.endpoint, endpoint)
  assert.equal(wait.diagnostics().phase, 'connected')
  assert.match(wait.diagnostics().stderrTail, /late diagnostic/)
  assert.equal(fake.listenerCount('error'), 0)
  assert.equal(fake.listenerCount('exit'), 0)
  assert.equal(fake.stderr.listenerCount('data'), 1)
  wait.dispose()
  assert.equal(fake.stderr.listenerCount('data'), 0)
  assert(scheduled.cleared() >= 1)
}

{
  const diagnostics = {
    phase: 'timeout' as const, elapsedMs: 10_000, endpointSeen: false,
    stderrBytes: 0, stderrTail: '', stderrTruncated: false, incompleteLineBytes: 0,
  }
  const original = new VisibilityStartupError('startup', diagnostics)
  const cleanup = new Error('cleanup')
  const combined = new AggregateError([original, cleanup], 'both')
  assert.equal(combined.errors[0], original)
  assert.equal(combined.errors[1], cleanup)
  assert.equal(startupDiagnosticsFrom(combined), diagnostics)
}

console.log('visibility startup stream checks: PASS')
