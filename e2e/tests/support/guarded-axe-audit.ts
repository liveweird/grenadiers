import AxeBuilder from '@axe-core/playwright'
import type { Frame, Page } from '@playwright/test'

type AxeAuditResult = Awaited<ReturnType<AxeBuilder['analyze']>>
export type LegacyAxeAudit = (page: Page) => Promise<AxeAuditResult>

function requireSingleFrame(frames: Frame[], boundary: 'before' | 'after'): void {
  if (frames.length !== 1) {
    throw new Error(`Reconnect accessibility audit requires one frame ${boundary} analysis`)
  }
}

const runLegacyAudit: LegacyAxeAudit = async (page) => new AxeBuilder({ page })
  .setLegacyMode(true)
  .analyze()

export async function runGuardedLegacyAxeAudit(
  page: Page,
  audit: LegacyAxeAudit = runLegacyAudit,
): Promise<AxeAuditResult> {
  let frameAttached = false
  let primaryFailed = false
  let primaryFailure: unknown
  let cleanupFailure: unknown
  let result: AxeAuditResult | undefined
  const markFrameAttached = (): void => { frameAttached = true }

  page.on('frameattached', markFrameAttached)
  try {
    requireSingleFrame(page.frames(), 'before')
    result = await audit(page)
    if (frameAttached) {
      throw new Error('Reconnect accessibility audit observed a frame attachment during analysis')
    }
    requireSingleFrame(page.frames(), 'after')
  } catch (error) {
    primaryFailed = true
    primaryFailure = error
  } finally {
    try {
      page.off('frameattached', markFrameAttached)
    } catch (error) {
      cleanupFailure = error
    }
  }
  if (primaryFailed) throw primaryFailure
  if (cleanupFailure) throw cleanupFailure
  if (!result) throw new Error('Reconnect accessibility audit returned no result')
  return result
}
