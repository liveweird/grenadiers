import { expect, test, type Browser, type Page } from '@playwright/test'

type Controller = 'human' | 'cpu'

async function createPages(browser: Browser, count: number): Promise<{ pages: Page[]; close: () => Promise<void> }> {
  const contexts = await Promise.all(Array.from({ length: count }, () => browser.newContext()))
  const pages = await Promise.all(contexts.map((context) => context.newPage()))
  await Promise.all(pages.map((page) => page.goto('/')))
  return { pages, close: async () => { await Promise.all(contexts.map((context) => context.close())) } }
}

async function createCustom(page: Page, controllers: readonly Controller[]): Promise<string | null> {
  await page.getByRole('combobox', { name: 'Players', exact: true }).selectOption(String(controllers.length))
  for (const [index, controller] of controllers.entries()) {
    if (index > 0) {
      await page.getByRole('combobox', { name: `Player ${index + 1} controller`, exact: true })
        .selectOption(controller)
    }
  }
  if (controllers.includes('cpu')) {
    await page.getByRole('combobox', { name: 'CPU difficulty', exact: true }).last().selectOption('standard')
  }
  await page.getByRole('button', { name: 'Create custom match', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Players', exact: true })).toBeVisible()
  if (!controllers.slice(1).includes('human')) return null
  const invitation = page.getByLabel('Room invitation code', { exact: true })
  await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  return invitation.innerText()
}

async function join(page: Page, code: string, invitationVisible: boolean): Promise<void> {
  await page.getByRole('textbox', { name: 'Room code', exact: true }).fill(code)
  await page.getByRole('button', { name: 'Join room', exact: true }).click()
  await expect(page.getByRole('list', { name: 'Players', exact: true })).toContainText('(you)')
  const invitation = page.getByLabel('Room invitation code', { exact: true })
  if (invitationVisible) await expect(invitation).toHaveText(code)
  else await expect(invitation).toHaveCount(0)
}

async function expectPlayers(page: Page, count: number): Promise<void> {
  const list = page.getByRole('list', { name: 'Players', exact: true })
  for (let index = 1; index <= count; index += 1) {
    await expect(list).toContainText(`Player ${['one', 'two', 'three', 'four'][index - 1]}`)
  }
  expect(await page.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
}

test('three humans join an explicit three-seat match', async ({ browser }) => {
  const group = await createPages(browser, 3)
  try {
    const code = await createCustom(group.pages[0], ['human', 'human', 'human'])
    expect(code).not.toBeNull()
    await join(group.pages[1], code!, true)
    await join(group.pages[2], code!, true)
    await expectPlayers(group.pages[0], 3)
  } finally { await group.close() }
})

test('four humans share the four-seat lobby at 320 pixels', async ({ browser }) => {
  const group = await createPages(browser, 4)
  try {
    const code = await createCustom(group.pages[0], ['human', 'human', 'human', 'human'])
    expect(code).not.toBeNull()
    await join(group.pages[1], code!, true)
    await join(group.pages[2], code!, true)
    await join(group.pages[3], code!, true)
    await group.pages[3].setViewportSize({ width: 320, height: 844 })
    await expectPlayers(group.pages[3], 4)
  } finally { await group.close() }
})

test('human CPU human assigns both human seats without exposing a CPU join', async ({ browser }) => {
  const group = await createPages(browser, 2)
  try {
    const code = await createCustom(group.pages[0], ['human', 'cpu', 'human'])
    expect(code).not.toBeNull()
    await join(group.pages[1], code!, false)
    await expectPlayers(group.pages[0], 3)
    await expect(group.pages[0].getByText('CPU difficulty: Standard', { exact: true })).toBeVisible()
  } finally { await group.close() }
})

test('one human and three CPUs creates a ready four-controller lobby', async ({ browser }) => {
  const group = await createPages(browser, 1)
  try {
    const code = await createCustom(group.pages[0], ['human', 'cpu', 'cpu', 'cpu'])
    expect(code).toBeNull()
    await expect(group.pages[0].getByLabel('Room invitation code', { exact: true })).toHaveCount(0)
    await expectPlayers(group.pages[0], 4)
    await expect(group.pages[0].getByRole('button', { name: 'Ready', exact: true })).toBeEnabled()
  } finally { await group.close() }
})
