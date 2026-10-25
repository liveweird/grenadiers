import type { BrowserContext } from '@playwright/test'

export type BrowserPolicyObservation = { violations: string[] }

const DIRECTIVE_PATTERN = /^[a-z][a-z0-9-]{0,63}$/

export async function observeBrowserPolicy(
  context: BrowserContext,
): Promise<BrowserPolicyObservation> {
  const observation: BrowserPolicyObservation = { violations: [] }
  await context.exposeBinding('recordBrowserPolicyViolation', (_source, value: unknown) => {
    if (typeof value === 'string' && DIRECTIVE_PATTERN.test(value)) {
      observation.violations.push(value)
    }
  })
  await context.addInitScript(() => {
    document.addEventListener('securitypolicyviolation', (event) => {
      void window.recordBrowserPolicyViolation(event.effectiveDirective)
    })
  })
  return observation
}

declare global {
  interface Window {
    recordBrowserPolicyViolation: (effectiveDirective: string) => Promise<void>
  }
}
