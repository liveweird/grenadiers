import { expect, test } from '@playwright/test'
import {
  DelayedFifo, QueueOverflowError, type DelayScheduler,
} from '../tests/support/delayed-websocket'

class FakeTime implements DelayScheduler {
  private time = 0
  private nextId = 1
  private readonly timers = new Map<number, { due: number; callback: () => void }>()
  now = (): number => this.time
  setTimer = (callback: () => void, delayMs: number): number => {
    const id = this.nextId
    this.nextId += 1
    this.timers.set(id, { due: this.time + delayMs, callback })
    return id
  }
  clearTimer = (timer: unknown): void => { this.timers.delete(timer as number) }

  advanceTo(target: number): void {
    while (true) {
      const next = [...this.timers.entries()].sort((left, right) => left[1].due - right[1].due)[0]
      if (!next || next[1].due > target) break
      this.timers.delete(next[0])
      this.time = next[1].due
      next[1].callback()
    }
    this.time = target
  }

  count(): number { return this.timers.size }
}

test('the delay queue preserves FIFO order under cycling jitter', () => {
  const time = new FakeTime()
  const delivered: object[] = []
  const frames = [{ id: 1 }, { id: 2 }, { id: 3 }]
  const queue = new DelayedFifo<object>(50, [-10, 10, -10], (value) => delivered.push(value), {
    scheduler: time,
  })
  queue.push(frames[0], 2)
  time.advanceTo(5)
  queue.push(frames[1], 3)
  time.advanceTo(10)
  queue.push(frames[2], 4)
  expect(time.count()).toBe(1)
  time.advanceTo(39)
  expect(delivered).toEqual([])
  time.advanceTo(40)
  expect(delivered).toEqual([frames[0]])
  time.advanceTo(65)
  expect(delivered).toEqual(frames)
  expect(delivered[0]).toBe(frames[0])
  expect(queue.stats).toMatchObject({ delivered: 3, peakFrames: 3, peakBytes: 9 })
})

test('the delay queue rejects frame and byte overflow atomically', () => {
  const time = new FakeTime()
  const frameBound = new DelayedFifo(50, [0], () => undefined, {
    scheduler: time, maxFrames: 2, maxBytes: 100,
  })
  frameBound.push('one', 3)
  frameBound.push('two', 3)
  expect(() => frameBound.push('three', 5)).toThrow(QueueOverflowError)
  expect(frameBound.stats).toMatchObject({ peakFrames: 2, peakBytes: 6, delivered: 0 })

  const byteBound = new DelayedFifo(50, [0], () => undefined, {
    scheduler: new FakeTime(), maxBytes: 5,
  })
  byteBound.push(Buffer.from([1, 2, 3]), 3)
  expect(() => byteBound.push(Buffer.from([4, 5, 6]), 3)).toThrow(QueueOverflowError)
  expect(byteBound.stats).toMatchObject({ peakFrames: 1, peakBytes: 3, delivered: 0 })
})

test('canceling the delay queue drops pending frames and its sole timer', () => {
  const time = new FakeTime()
  const delivered: string[] = []
  const queue = new DelayedFifo<string>(50, [0], (value) => delivered.push(value), {
    scheduler: time,
  })
  queue.push('private-frame-not-retained', 26)
  queue.push('second', 6)
  expect(time.count()).toBe(1)
  queue.cancel()
  expect(time.count()).toBe(0)
  time.advanceTo(100)
  expect(delivered).toEqual([])
  expect(queue.stats.canceled).toBe(2)
  expect(() => queue.push('late', 4)).toThrow('Delay queue is closed')
})
