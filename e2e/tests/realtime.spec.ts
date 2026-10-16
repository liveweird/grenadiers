import AxeBuilder from '@axe-core/playwright'
import { expect, test } from '@playwright/test'

test('two browsers complete independent realtime roundtrips', async ({ browser }) => {
  const contexts = await Promise.all([browser.newContext(), browser.newContext()])
  try {
    const pages = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all(pages.map((page) => page.goto('/')))
    await Promise.all(pages.map((page) => expect(page.getByRole('heading', { name: 'Grenadiers' })).toBeVisible()))
    await Promise.all(pages.map((page) => expect(page.getByText('Available')).toBeVisible()))

    const disconnectedAccessibility = await new AxeBuilder({ page: pages[0] }).analyze()
    expect(disconnectedAccessibility.violations).toEqual([])

    await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Connect' }).click()))
    await Promise.all(pages.map((page) => expect(page.getByText('Protocol v1 · up to 2 players')).toBeVisible()))
    await Promise.all(pages.map((page) => page.getByRole('button', { name: 'Send ping' }).click()))
    await Promise.all(pages.map((page) => expect(page.getByText('Pong 1 received')).toBeVisible()))

    const accessibility = await new AxeBuilder({ page: pages[0] }).analyze()
    expect(accessibility.violations).toEqual([])
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
