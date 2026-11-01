import { createHash } from 'node:crypto'
import { expect, type BrowserContext, type Page, type TestInfo } from '@playwright/test'
import type { GameServerMessage, Seat } from '../../web/src/game/v19/schema'
import { expectPaintedCanvas, latestSnapshot, observeGame, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { installTransportProbe } from '../tests/support/reconnect'
import { applyItemSettings } from '../tests/support/settings'
import { markStage, type Cell, type LiveWindowDiagnostics, type PixelComparison,
  type TaskOutcome } from './live-pixel-window-control'
export { markStage, recordDetonatedBomb, recordPixelComparison, recordPlacedBomb, requireSuccessfulTasks,
  settleAll, validateLiveGuard } from './live-pixel-window-control'
export type { LiveWindowDiagnostics, PixelComparison, TaskOutcome } from './live-pixel-window-control'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
export type PreparedTileCapture = { column: number; row: number;
  viewport: { width: number; height: number };
  clip: { x: number; y: number; width: number; height: number } }
export type LivePng = { name: string; body: Buffer; attached: boolean }

async function tileCaptureGeometry(page: Page, column: number, row: number,
  scroll: boolean): Promise<PreparedTileCapture> {
  const canvas = page.locator('.game-arena canvas')
  if (scroll) await canvas.scrollIntoViewIfNeeded()
  const [box, viewport] = await Promise.all([canvas.boundingBox(), Promise.resolve(page.viewportSize())])
  if (!box || !viewport) throw new Error('arena canvas bounds are unavailable')
  const left = Math.ceil(box.x + column * box.width / 15)
  const top = Math.ceil(box.y + row * box.height / 13)
  const right = Math.floor(box.x + (column + 1) * box.width / 15)
  const bottom = Math.floor(box.y + (row + 1) * box.height / 13)
  if (left < 0 || top < 0 || right > viewport.width || bottom > viewport.height
    || right <= left || bottom <= top) throw new Error('arena tile clip is outside the viewport')
  return { column, row, viewport,
    clip: { x: left, y: top, width: right - left, height: bottom - top } }
}

export function prepareTileCapture(page: Page, column: number, row: number) {
  return tileCaptureGeometry(page, column, row, true)
}

export async function capturePreparedTilePng(page: Page,
  prepared: PreparedTileCapture): Promise<Buffer> {
  const current = await tileCaptureGeometry(page, prepared.column, prepared.row, false)
  if (JSON.stringify(current) !== JSON.stringify(prepared)) {
    throw new Error('arena tile capture geometry changed during the live interval')
  }
  return page.screenshot({ clip: prepared.clip, scale: 'css' })
}

export async function retainPng(pngs: LivePng[], name: string,
  capture: Promise<Buffer>): Promise<Buffer> {
  const body = await capture
  pngs.push({ name, body, attached: false })
  return body
}

export async function prepareInvisibleBombsMatch(contexts: BrowserContext[], info: TestInfo,
  plant: (page: Page, actor: ObservedGame, sighted: ObservedGame,
    cell: Cell) => Promise<unknown>): Promise<{
      one: Page; two: Page; games: [ObservedGame, ObservedGame];
      prepared: [PreparedTileCapture, PreparedTileCapture]
    }> {
  const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
  await Promise.all([one, two].map(installTransportProbe))
  const games: [ObservedGame, ObservedGame] = [observeGame(one), observeGame(two)]
  await Promise.all([one.goto('/'), two.goto('/')])
  await one.getByRole('button', { name: 'Create room', exact: true }).click()
  const code = one.getByLabel('Room invitation code', { exact: true })
  await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
  await applyItemSettings(one, games[0], [{ id: 'invisibleBombs', label: 'Invisible bombs' }],
    () => one.getByLabel('Bomb fuse').selectOption('300'))
  await two.getByText('Polski', { exact: true }).click()
  await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
  await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
  await Promise.all([one.getByRole('button', { name: 'Ready', exact: true }).click(),
    two.getByRole('button', { name: 'Gotowy', exact: true }).click()])
  await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type)
    .toBe('running')))
  await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
  await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()
  await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
  await plant(one, games[0], games[0], { column: 2, row: 1 })
  await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
  await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
  await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length
    + latestSnapshot(games[0])!.state.flames.length).toBe(0)
  await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
    cell: { column: 3, row: 1 }, kind: 'invisibleBombs' })
  await Promise.all([one, two].map(expectPaintedCanvas))
  await one.screenshot({ path: info.outputPath('invisible-bombs-before-collection-desktop-en.png'), fullPage: true })
  expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
  await two.screenshot({ path: info.outputPath('invisible-bombs-before-collection-polish-320-reduced.png'), fullPage: true })
  const prepared = await Promise.all([prepareTileCapture(one, 3, 1),
    prepareTileCapture(two, 3, 1)]) as [PreparedTileCapture, PreparedTileCapture]
  await moveSeatTo(two, games[1], 'two', { column: 12, row: 11 })
  await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
  await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
  return { one, two, games, prepared }
}

export async function trackTask<T>(name: string, diagnostics: LiveWindowDiagnostics,
  operation: () => Promise<T>): Promise<TaskOutcome<T>> {
  markStage(diagnostics, name, 'started')
  try {
    const value = await operation()
    markStage(diagnostics, name, 'passed')
    return { name, ok: true, value }
  } catch (error) {
    markStage(diagnostics, name, 'failed')
    return { name, ok: false, error }
  }
}

function effectTicks(state: Snapshot['state'], seat: Seat): number {
  return state.actors.find((actor): actor is Extract<Snapshot['state']['actors'][number],
    { type: 'visible' }> => actor.type === 'visible' && actor.seat === seat)?.effects
    .find(({ kind }) => kind === 'invisibleBombs')?.remainingTicks ?? 0
}

function publicWindowReceipt(games: ObservedGame[], selectedBombIds: number[],
  effectSeat: Seat, since: number): unknown {
  const selected = new Set(selectedBombIds)
  return games.map((game, observer) => ({
    observer,
    frames: game.frames.filter((frame): frame is Snapshot => frame.type === 'snapshot'
      && frame.state.physicalTick >= since).slice(-64).map((frame) => ({
      roundId: frame.room.roundId,
      physicalTick: frame.state.physicalTick,
      effectTicks: effectTicks(frame.state, effectSeat),
      bombs: frame.state.bombs.filter(({ id }) => selected.has(id)).map((bomb) => ({
        id: bomb.id, cell: bomb.cell, trigger: bomb.trigger.type === 'timed'
          ? { type: 'timed' as const, remainingTicks: bomb.trigger.remainingTicks }
          : { type: bomb.trigger.type },
      })),
      events: frame.events.flatMap(({ tick, kind }) => (kind.type === 'bombPlaced'
        || kind.type === 'bombDetonated') && selected.has(kind.bomb.id)
        ? [{ tick, type: kind.type, bombId: kind.bomb.id, cell: kind.bomb.cell }] : []),
    })),
  }))
}

export async function attachLiveWindowEvidence(input: { info: TestInfo; failed: boolean;
  diagnostics: LiveWindowDiagnostics; livePngs: LivePng[]; pixelComparisons: PixelComparison[];
  games: ObservedGame[]; receiptStart: number }): Promise<void> {
  const { info, failed, diagnostics, livePngs, pixelComparisons, games, receiptStart } = input
  const artifacts = livePngs.map(({ name, body }) => ({ name,
    sha256: createHash('sha256').update(body).digest('hex'), bytes: body.length }))
  const receipt = { diagnostics, artifacts, pixelComparisons,
    publicFrames: games.length === 2 ? publicWindowReceipt(games,
      diagnostics.selectedBombs.map(({ id }) => id), 'one', receiptStart) : [] }
  const attachments: { name: string; body: Buffer; contentType: string; png?: LivePng }[] = [{
    name: 'invisible-bombs-live-window.json', body: Buffer.from(JSON.stringify(receipt, null, 2)),
    contentType: 'application/json',
  }]
  if (failed) attachments.push(...livePngs.map((png) => ({
    name: png.name, body: png.body, contentType: 'image/png', png,
  })))
  const errors: unknown[] = []
  for (const attachment of attachments) try {
    await info.attach(attachment.name, { body: attachment.body, contentType: attachment.contentType })
    if (attachment.png) attachment.png.attached = true
  } catch (error) { errors.push(error) }
  if (errors.length > 0) {
    info.annotations.push({ type: 'live-window-evidence', description: `${errors.length}-attachment-failures` })
    if (!failed) throw new AggregateError(errors, 'Live-window evidence attachment failed')
  }
}
