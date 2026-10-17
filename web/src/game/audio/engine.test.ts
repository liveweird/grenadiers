import { describe, expect, it, vi } from 'vitest'
import { GameAudio } from './engine'

function deferred() {
  let resolve!: () => void
  let reject!: (error: Error) => void
  const promise = new Promise<void>((yes, no) => { resolve = yes; reject = no })
  return { promise, resolve, reject }
}

function fakeContext(resumePromise?: Promise<void>) {
  const oscillators: Array<{
    onended: (() => void) | null; start: ReturnType<typeof vi.fn>; stop: ReturnType<typeof vi.fn>
    disconnect: ReturnType<typeof vi.fn>
  }> = []
  const gains: Array<{ disconnect: ReturnType<typeof vi.fn> }> = []
  const parameter = () => ({ setValueAtTime: vi.fn(), exponentialRampToValueAtTime: vi.fn() })
  const context = {
    state: 'suspended', currentTime: 1, destination: {},
    resume: vi.fn(async () => {
      await resumePromise
      context.state = 'running'
    }),
    suspend: vi.fn(async () => { context.state = 'suspended' }),
    close: vi.fn(async () => { context.state = 'closed' }),
    createGain: vi.fn(() => {
      const gain = { gain: parameter(), connect: vi.fn(), disconnect: vi.fn() }
      gains.push(gain)
      return gain
    }),
    createOscillator: vi.fn(() => {
      const oscillator = {
        type: 'sine', frequency: parameter(), connect: vi.fn(), disconnect: vi.fn(),
        start: vi.fn(), stop: vi.fn(), onended: null as (() => void) | null,
      }
      oscillators.push(oscillator)
      return oscillator
    }),
  }
  return { context: context as unknown as AudioContext, raw: context, oscillators, gains }
}

describe('procedural audio engine', () => {
  it('constructs only from a gesture and bounds every scheduled voice', async () => {
    const fake = fakeContext()
    const create = vi.fn(() => fake.context)
    const audio = new GameAudio(create)
    expect(create).not.toHaveBeenCalled()
    expect(await audio.enableFromGesture()).toBe(true)
    audio.play(['placement', 'detonation', 'pickup', 'elimination', 'placement'])
    expect(fake.oscillators).toHaveLength(4)
    expect(fake.oscillators.every((voice) => voice.start.mock.calls.length === 1
      && voice.stop.mock.calls.length === 1)).toBe(true)
    audio.silence()
    expect(fake.oscillators.every((voice) => voice.disconnect.mock.calls.length === 1)).toBe(true)
    expect(fake.raw.suspend).toHaveBeenCalledOnce()
  })

  it('rejects a failed resume and ignores an enable that finishes after silence', async () => {
    const failed = fakeContext(Promise.reject(new Error('denied')))
    expect(await new GameAudio(() => failed.context).enableFromGesture()).toBe(false)
    const pending = deferred()
    const late = fakeContext(pending.promise)
    const audio = new GameAudio(() => late.context)
    const enabling = audio.enableFromGesture()
    audio.silence()
    pending.resolve()
    expect(await enabling).toBe(false)
    audio.play(['placement'])
    expect(late.oscillators).toHaveLength(0)
  })

  it('stops and disconnects voices and closes once on disposal', async () => {
    const fake = fakeContext()
    const audio = new GameAudio(() => fake.context)
    await audio.enableFromGesture()
    audio.play(['placement'])
    audio.dispose()
    audio.dispose()
    expect(fake.oscillators[0].stop).toHaveBeenCalled()
    expect(fake.gains.every((gain) => gain.disconnect.mock.calls.length === 1)).toBe(true)
    expect(fake.raw.close).toHaveBeenCalledOnce()
  })

  it('cannot let a disposed pending resume enable a replacement generation', async () => {
    const pending = deferred()
    const stale = fakeContext(pending.promise)
    const disposed = new GameAudio(() => stale.context)
    const staleEnable = disposed.enableFromGesture()
    disposed.dispose()

    const fresh = fakeContext()
    const replacement = new GameAudio(() => fresh.context)
    expect(await replacement.enableFromGesture()).toBe(true)
    pending.resolve()
    expect(await staleEnable).toBe(false)
    disposed.play(['placement'])
    replacement.play(['placement'])
    expect(stale.oscillators).toHaveLength(0)
    expect(fresh.oscillators).toHaveLength(1)
  })
})
