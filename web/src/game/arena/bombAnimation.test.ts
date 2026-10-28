import { describe, expect, it } from 'vitest'
import { selectBombAnimation } from './bombAnimation'

describe('bomb animation selection', () => {
  it('uses discrete bounded pulses in three fuse urgency bands', () => {
    expect(selectBombAnimation(180, true)).toEqual({ shellScale: 1.125, sparkVisible: true })
    expect(selectBombAnimation(195, true)).toEqual({ shellScale: 1, sparkVisible: false })
    expect(selectBombAnimation(120, true)).toEqual({ shellScale: 1.125, sparkVisible: true })
    expect(selectBombAnimation(136, true)).toEqual({ shellScale: 1, sparkVisible: false })
    expect(selectBombAnimation(60, true)).toEqual({ shellScale: 1.125, sparkVisible: true })
    expect(selectBombAnimation(68, true)).toEqual({ shellScale: 1, sparkVisible: false })
  })

  it('switches cadence exactly at the 120 and 60 tick boundaries', () => {
    expect(selectBombAnimation(121, true).shellScale).toBe(1.125)
    expect(selectBombAnimation(120, true).shellScale).toBe(1.125)
    expect(selectBombAnimation(61, true).shellScale).toBe(1.125)
    expect(selectBombAnimation(60, true).shellScale).toBe(1.125)
  })

  it('increases pulse frequency as authoritative fuse time shortens', () => {
    const pulseStarts = (from: number, to: number): number => {
      let starts = 0
      let previous = false
      for (let ticks = from; ticks >= to; ticks -= 1) {
        const pulsing = selectBombAnimation(ticks, true).shellScale > 1
        if (pulsing && !previous) starts += 1
        previous = pulsing
      }
      return starts
    }
    expect([
      pulseStarts(179, 121), pulseStarts(119, 61), pulseStarts(59, 1),
    ]).toEqual([1, 2, 5])
  })

  it('is static when animation is disabled or no timed countdown exists', () => {
    expect(selectBombAnimation(60, false)).toEqual({ shellScale: 1, sparkVisible: true })
    expect(selectBombAnimation(null, true)).toEqual({ shellScale: 1, sparkVisible: true })
  })

  it('clamps expired and fractional authoritative values deterministically', () => {
    expect(selectBombAnimation(-10, true)).toEqual({ shellScale: 1.125, sparkVisible: true })
    expect(selectBombAnimation(12.9, true)).toEqual({ shellScale: 1.125, sparkVisible: true })
  })
})
