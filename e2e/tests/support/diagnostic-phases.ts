import type { Page } from '@playwright/test'
import type { PagePhaseLabel } from './page-stall-collector'

type PhaseWindow = Window & {
  grenadiersStallDiagnosticPhase?: (label: PagePhaseLabel) => void
}

export async function readViewportFits(page: Page): Promise<boolean> {
  return page.evaluate(() => {
    const phase = (window as PhaseWindow).grenadiersStallDiagnosticPhase
    phase?.('page-enter')
    phase?.('read-start')
    const fits = document.documentElement.scrollWidth <= innerWidth
    phase?.('read-end')
    return fits
  })
}
