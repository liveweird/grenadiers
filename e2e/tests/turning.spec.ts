import { expect, test } from '@playwright/test'
import { visibleActor, latestSnapshot, observeGame } from './support/duel'

test('native keyboard turns recover from a wall stop outside the legacy alignment window', async ({ browser }) => {
  const contexts = await Promise.all([browser.newContext(), browser.newContext()])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one, two].map((page) => page.goto('/')))
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const invitation = one.getByLabel('Room invitation code', { exact: true })
    await expect(invitation).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await two.getByRole('textbox', { name: 'Room code', exact: true }).fill(await invitation.innerText())
    await two.getByRole('button', { name: 'Join room', exact: true }).click()
    await one.getByRole('button', { name: 'Ready', exact: true }).click()
    await two.getByRole('button', { name: 'Ready', exact: true }).click()
    await expect.poll(() => latestSnapshot(games[0])?.room.phase.type).toBe('running')
    expect(latestSnapshot(games[0])!.room.rulesVersion).toBe(4)

    const actor = (index: number) => visibleActor(latestSnapshot(games[index]), 'one')
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await one.keyboard.down('ArrowLeft')
    // The border stops the footprint at 1,344: 192 units short of center.
    // This lies outside the old 128-unit window and inside the new 256-unit window.
    await expect.poll(() => actor(0)?.position.x).toBe(1344)
    await one.keyboard.up('ArrowLeft')
    await one.keyboard.down('ArrowDown')
    await expect.poll(() => actor(0)?.position.y).toBeGreaterThan(1800)
    await one.keyboard.up('ArrowDown')
    await expect.poll(() => actor(1)?.position.y).toBeGreaterThan(1800)
    for (const index of [0, 1]) {
      expect(actor(index)?.position.x).toBe(1536)
      expect(actor(index)?.heading).toBe('south')
      expect(actor(index)?.alive).toBe(true)
    }
  } finally {
    await Promise.all(contexts.map((context) => context.close()))
  }
})
