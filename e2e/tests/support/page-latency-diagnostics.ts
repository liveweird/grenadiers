import type { Page } from '@playwright/test'

type DomKeyTiming = { type: 'keydown' | 'keyup'; key: string; at: number }
export type DomClockDiagnostics = { timeOrigin: number; timings: DomKeyTiming[] }

type TimingWindow = Window & { grenadiersKeyTimings?: DomKeyTiming[] }

export async function installDomKeyTimings(page: Page): Promise<void> {
  await page.addInitScript(() => {
    const observed = window as TimingWindow
    observed.grenadiersKeyTimings = []
    for (const type of ['keydown', 'keyup'] as const) {
      addEventListener(type, (event) => {
        const timings = observed.grenadiersKeyTimings
        timings?.push({ type, key: event.key, at: performance.now() })
        if (timings && timings.length > 1_024) timings.shift()
      })
    }
  })
}

export async function domClockDiagnostics(page: Page): Promise<DomClockDiagnostics> {
  return page.evaluate(() => ({
    timeOrigin: performance.timeOrigin,
    timings: (window as TimingWindow).grenadiersKeyTimings ?? [],
  }))
}
