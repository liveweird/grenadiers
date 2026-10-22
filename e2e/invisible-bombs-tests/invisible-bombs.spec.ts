import { expect, test, type Page } from '@playwright/test'
import { expectPaintedCanvas, latestSnapshot, observeGame, visibleActor, type ObservedGame } from '../tests/support/duel'
import { moveSeatTo } from '../tests/support/movement'
import { admissionBaseline, disconnectTransport, installTransportProbe } from '../tests/support/reconnect'
import type { GameServerMessage, Seat } from '../../web/src/game/v9/schema'

test.beforeAll(() => expect(process.env.E2E_INVISIBLE_BOMBS_FIXTURE).toBe('arena-v2-invisible-bombs'))

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type Admission = Extract<GameServerMessage, { type: 'admitted' }>
type StatefulAdmission = Admission & { state: NonNullable<Admission['state']> }
type VisibleActor = Extract<Snapshot['state']['actors'][number], { type: 'visible' }>
type Bomb = Snapshot['state']['bombs'][number]
type ReconnectProofInput = {
  deadline: number
  games: ObservedGame[]
  one: Page
  ownBomb: Bomb
  privacyStart: number
  rivalBomb: Bomb
  two: Page
}
const snapshots = (game: ObservedGame): Snapshot[] => game.frames.filter(
  (frame): frame is Snapshot => frame.type === 'snapshot')
const stateEffectTicks = (state: Snapshot['state'], seat: Seat) => state.actors
  .find((actor): actor is VisibleActor => actor.type === 'visible' && actor.seat === seat)?.effects
  .find(({ kind }) => kind === 'invisibleBombs')?.remainingTicks ?? 0
const effectTicks = (game: ObservedGame, seat: Seat) =>
  stateEffectTicks(latestSnapshot(game)!.state, seat)
const bombEvents = (game: ObservedGame, since: number) => snapshots(game)
  .filter(({ state }) => state.physicalTick >= since).flatMap(({ events }) => events)
  .filter(({ kind }) => kind.type === 'bombPlaced' || kind.type === 'bombDetonated')

async function captureTilePng(page: Page, column: number, row: number): Promise<Buffer> {
  const canvas = page.locator('.game-arena canvas')
  await canvas.scrollIntoViewIfNeeded()
  const [box, viewport] = await Promise.all([canvas.boundingBox(), Promise.resolve(page.viewportSize())])
  if (!box || !viewport) throw new Error('arena canvas bounds are unavailable')
  const left = Math.ceil(box.x + column * box.width / 15)
  const top = Math.ceil(box.y + row * box.height / 13)
  const right = Math.floor(box.x + (column + 1) * box.width / 15)
  const bottom = Math.floor(box.y + (row + 1) * box.height / 13)
  if (left < 0 || top < 0 || right > viewport.width || bottom > viewport.height
    || right <= left || bottom <= top) throw new Error('arena tile clip is outside the viewport')
  return page.screenshot({
    clip: { x: left, y: top, width: right - left, height: bottom - top }, scale: 'css',
  })
}

async function decodeTilePng(page: Page, png: Buffer): Promise<string[]> {
  return page.evaluate(async (encoded) => {
    const image = new Image(); image.src = `data:image/png;base64,${encoded}`; await image.decode()
    const canvas = document.createElement('canvas'); canvas.width = image.width; canvas.height = image.height
    const context = canvas.getContext('2d'); if (!context) throw new Error('decoder unavailable')
    context.drawImage(image, 0, 0)
    const pixels = context.getImageData(0, 0, image.width, image.height).data
    const result: string[] = []
    for (let index = 0; index < pixels.length; index += 16) {
      result.push(`${pixels[index]},${pixels[index + 1]},${pixels[index + 2]},${pixels[index + 3]}`)
    }
    return result
  }, png.toString('base64'))
}

function expectLivePixelWindow(blinded: ObservedGame, sighted: ObservedGame,
  bombId: number, deadline: number): number {
  const hidden = latestSnapshot(blinded)!; const visible = latestSnapshot(sighted)!
  expect([hidden.room.roomId, hidden.room.roundId]).toEqual([visible.room.roomId, visible.room.roundId])
  expect(hidden.state.arena.terrain[1 * 15 + 3]).toBe('floor')
  expect(visible.state.arena.terrain[1 * 15 + 3]).toBe('floor')
  expect(hidden.state.physicalTick + stateEffectTicks(hidden.state, 'one')).toBe(deadline)
  expect(hidden.state.bombs).toEqual([])
  const bomb = visible.state.bombs.find(({ id }) => id === bombId)
  expect(bomb?.trigger.type).toBe('timed')
  if (bomb?.trigger.type !== 'timed') throw new Error('own compositor bomb must be Timed')
  expect(bomb.trigger.remainingTicks).toBeGreaterThan(0)
  expect(visible.state.flames).not.toContainEqual(expect.objectContaining({ cell: { column: 3, row: 1 } }))
  expect(visible.state.actors.some((actor) => actor.type === 'visible'
    && Math.floor(actor.position.x / 1024) === 3 && Math.floor(actor.position.y / 1024) === 1)).toBe(false)
  return visible.state.physicalTick
}

function expectRevealedPixelWindow(game: ObservedGame, bombId: number,
  deadline: number, roundId: number): number {
  const frame = latestSnapshot(game)!
  expect(frame.room.roundId).toBe(roundId)
  expect(frame.state.physicalTick).toBeGreaterThanOrEqual(deadline)
  expect(stateEffectTicks(frame.state, 'two')).toBe(0)
  expect(frame.state.arena.terrain[11 * 15 + 12]).toBe('floor')
  const bomb = frame.state.bombs.find(({ id }) => id === bombId)
  expect(bomb?.cell).toEqual({ column: 12, row: 11 })
  expect(bomb?.trigger.type).toBe('timed')
  if (bomb?.trigger.type !== 'timed') throw new Error('revealed compositor bomb must be Timed')
  expect(bomb.trigger.remainingTicks).toBeGreaterThan(0)
  expect(frame.state.flames).not.toContainEqual(expect.objectContaining({ cell: { column: 12, row: 11 } }))
  expect(frame.state.actors.some((actor) => actor.type === 'visible'
    && Math.floor(actor.position.x / 1024) === 12 && Math.floor(actor.position.y / 1024) === 11)).toBe(false)
  return frame.state.physicalTick
}

async function plant(page: Page, actorGame: ObservedGame, sightedGame: ObservedGame,
  cell: { column: number; row: number }) {
  const beforeFrame = latestSnapshot(sightedGame)!
  const before = new Set(beforeFrame.state.bombs.map(({ id }) => id))
  const actionFloor = actorGame.commands.length
  await page.keyboard.press('Space')
  await expect.poll(() => actorGame.commands.slice(actionFloor)).toEqual([
    expect.objectContaining({ type: 'manualAction', action: 'bomb' }),
  ])
  await expect.poll(() => latestSnapshot(sightedGame)!.state.bombs.find((bomb) =>
    !before.has(bomb.id) && bomb.cell.column === cell.column && bomb.cell.row === cell.row)).toBeTruthy()
  const frame = latestSnapshot(sightedGame)!
  expect(frame.room.roundId).toBe(beforeFrame.room.roundId)
  expect(frame.room.revision).toBeGreaterThan(beforeFrame.room.revision)
  expect(frame.room.tick).toBeGreaterThanOrEqual(beforeFrame.room.tick)
  return frame.state.bombs.find((bomb) => !before.has(bomb.id))!
}

async function reconnectAndCaptureLiveBombs({ deadline, games, one, ownBomb, privacyStart,
  rivalBomb, two }: ReconnectProofInput) {
  const admissionFrameFloor = games[0].frames.length
  const beforeReconnect = latestSnapshot(games[0])!
  await disconnectTransport(one)
  await expect.poll(async () => (await admissionBaseline(one))?.tick).toBeGreaterThan(privacyStart)
  await expect.poll(() => games[0].frames.slice(admissionFrameFloor).some((frame) =>
    frame.type === 'admitted' && frame.room.roomId === beforeReconnect.room.roomId
    && frame.room.roundId === beforeReconnect.room.roundId && frame.youSeat === 'one'
    && frame.room.revision > beforeReconnect.room.revision && frame.state !== null)).toBe(true)
  const resumed = games[0].frames.slice(admissionFrameFloor).findLast((frame): frame is StatefulAdmission =>
    frame.type === 'admitted' && frame.room.roomId === beforeReconnect.room.roomId
    && frame.room.roundId === beforeReconnect.room.roundId && frame.youSeat === 'one'
    && frame.room.revision > beforeReconnect.room.revision && frame.state !== null)!
  const resumedEffectTicks = stateEffectTicks(resumed.state, 'one')
  expect(resumed.state.physicalTick).toBeLessThan(deadline)
  expect(resumedEffectTicks).toBeGreaterThan(0)
  expect(resumed.state.physicalTick + resumedEffectTicks).toBe(deadline)
  expect(resumed.state.bombs).toEqual([])
  expect('events' in resumed).toBe(false)
  const sightedFrameFloor = games[1].frames.length
  const admissionIndex = games[0].frames.findLastIndex((frame) => frame === resumed)
  await expect.poll(() => games[0].frames.slice(admissionIndex + 1).some((frame) =>
    frame.type === 'snapshot' && frame.room.roomId === resumed.room.roomId
    && frame.room.roundId === resumed.room.roundId && frame.state.physicalTick >= resumed.state.physicalTick
    && stateEffectTicks(frame.state, 'one') > 0
    && frame.state.physicalTick + stateEffectTicks(frame.state, 'one') === deadline
    && frame.state.bombs.length === 0)).toBe(true)
  await expect.poll(() => games[1].frames.slice(sightedFrameFloor).some((frame) =>
    frame.type === 'snapshot' && frame.room.roomId === resumed.room.roomId
    && frame.room.roundId === resumed.room.roundId && frame.state.physicalTick >= resumed.state.physicalTick
    && frame.state.bombs.some((bomb) => bomb.id === ownBomb.id && bomb.trigger.type === 'timed'
      && bomb.trigger.remainingTicks > 0))).toBe(true)
  expect(latestSnapshot(games[0])!.state.physicalTick).toBeLessThan(deadline)
  expect(effectTicks(games[0], 'one')).toBeGreaterThan(0)
  expect(latestSnapshot(games[0])!.state.bombs).toEqual([])
  const connectedBomb = latestSnapshot(games[1])!.state.bombs.find(({ id }) =>
    id === ownBomb.id || id === rivalBomb.id)
  expect(connectedBomb).toBeDefined()
  expect(connectedBomb?.trigger.type).toBe('timed')
  if (connectedBomb?.trigger.type !== 'timed') throw new Error('live reconnect bomb must be Timed')
  expect(connectedBomb.trigger.remainingTicks).toBeGreaterThan(0)
  expect(bombEvents(games[0], privacyStart)).toHaveLength(0)
  const captureStart = expectLivePixelWindow(games[0], games[1], ownBomb.id, deadline)
  const [hiddenPng, sightedPng] = await Promise.all([
    captureTilePng(one, 3, 1), captureTilePng(two, 3, 1),
  ])
  const captureEnd = expectLivePixelWindow(games[0], games[1], ownBomb.id, deadline)
  return {
    admissionTick: resumed.state.physicalTick,
    captureTicks: [captureStart, captureEnd],
    disconnectTick: beforeReconnect.state.physicalTick,
    hiddenPng,
    sightedPng,
  }
}

test('Invisible Bombs hides live bombs and cues until exact expiry', async ({ browser }, info) => {
  const contexts = await Promise.all([
    browser.newContext({ viewport: { width: 1440, height: 1050 } }),
    browser.newContext({ viewport: { width: 320, height: 844 }, reducedMotion: 'reduce' }),
  ])
  try {
    const [one, two] = await Promise.all(contexts.map((context) => context.newPage()))
    await Promise.all([one, two].map(installTransportProbe))
    const games = [observeGame(one), observeGame(two)]
    await Promise.all([one.goto('/'), two.goto('/')])
    await one.getByRole('button', { name: 'Create room', exact: true }).click()
    const code = one.getByLabel('Room invitation code', { exact: true })
    await expect(code).toHaveText(/^[A-HJ-NP-Z2-9]{8}$/)
    await one.getByRole('checkbox', { name: 'Heart', exact: true }).uncheck()
    await one.getByRole('checkbox', { name: 'Invisible bombs', exact: true }).check()
    await one.getByLabel('Bomb fuse').selectOption('300')
    await one.getByRole('button', { name: 'Apply rules', exact: true }).click()
    await two.getByText('Polski', { exact: true }).click()
    await two.getByRole('textbox', { name: 'Kod pokoju', exact: true }).fill(await code.innerText())
    await two.getByRole('button', { name: 'Dołącz do pokoju', exact: true }).click()
    await Promise.all([one.getByRole('button', { name: 'Ready', exact: true }).click(),
      two.getByRole('button', { name: 'Gotowy', exact: true }).click()])
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)?.room.phase.type).toBe('running')))
    await one.getByRole('region', { name: 'Grenadiers courtyard' }).click()
    await two.getByRole('region', { name: 'Dziedziniec Grenadiers', exact: true }).click()
    await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 })
    await plant(one, games[0], games[0], { column: 2, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 }); await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 })
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs.length + latestSnapshot(games[0])!.state.flames.length).toBe(0)
    await expect.poll(() => latestSnapshot(games[0])!.state.pickups).toContainEqual({
      cell: { column: 3, row: 1 }, kind: 'invisibleBombs' })
    await Promise.all([one, two].map(expectPaintedCanvas))
    await one.screenshot({ path: info.outputPath('invisible-bombs-before-collection-desktop-en.png'), fullPage: true })
    expect(await two.evaluate(() => document.documentElement.scrollWidth > innerWidth)).toBe(false)
    await two.screenshot({ path: info.outputPath('invisible-bombs-before-collection-polish-320-reduced.png'), fullPage: true })

    await moveSeatTo(two, games[1], 'two', { column: 12, row: 11 })
    await moveSeatTo(one, games[0], 'one', { column: 1, row: 1 })
    await moveSeatTo(one, games[0], 'one', { column: 3, row: 1 })
    await expect.poll(() => effectTicks(games[0], 'one')).toBeGreaterThan(0)
    const collection = snapshots(games[0]).flatMap(({ events }) => events).find(({ kind }) =>
      kind.type === 'pickupCollected' && kind.seat === 'one' && kind.kind === 'invisibleBombs')!
    const deadline = collection.tick + 600
    const privacyStart = latestSnapshot(games[0])!.state.physicalTick
    const ownBomb = await plant(one, games[0], games[1], { column: 3, row: 1 })
    const rivalBomb = await plant(two, games[1], games[1], { column: 12, row: 11 })
    await Promise.all([
      (async () => { await moveSeatTo(one, games[0], 'one', { column: 2, row: 1 }); await moveSeatTo(one, games[0], 'one', { column: 1, row: 2 }) })(),
      (async () => { await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 }); await moveSeatTo(two, games[1], 'two', { column: 13, row: 10 }) })(),
    ])
    expect(ownBomb.placedAt + 300).toBeLessThan(deadline)
    await expect.poll(() => latestSnapshot(games[0])!.state.bombs).toEqual([])
    expect(visibleActor(latestSnapshot(games[0]), 'one')?.liveBombCount).toBe(1)
    await expect.poll(() => latestSnapshot(games[1])!.state.bombs.map(({ id }) => id))
      .toEqual(expect.arrayContaining([ownBomb.id, rivalBomb.id]))
    expect(bombEvents(games[0], privacyStart)).toHaveLength(0)
    const reconnect = await reconnectAndCaptureLiveBombs({
      deadline, games, one, ownBomb, privacyStart, rivalBomb, two,
    })
    await expect.poll(() => bombEvents(games[1], privacyStart).some(({ tick, kind }) =>
      tick < deadline && kind.type === 'bombDetonated' && (kind.bomb.id === ownBomb.id
        || kind.bomb.id === rivalBomb.id))).toBe(true)
    const witnessedDetonation = bombEvents(games[1], privacyStart).find(({ tick, kind }) =>
      tick < deadline && kind.type === 'bombDetonated' && (kind.bomb.id === ownBomb.id
        || kind.bomb.id === rivalBomb.id))!
    info.annotations.push({ type: 'phase-physical-ticks', description: JSON.stringify({
      collection: collection.tick, ownPlacement: ownBomb.placedAt, rivalPlacement: rivalBomb.placedAt,
      disconnect: reconnect.disconnectTick, admission: reconnect.admissionTick,
      captures: reconnect.captureTicks, detonation: witnessedDetonation.tick,
    }) })
    await expect.poll(() => snapshots(games[0]).some(({ state }) => state.physicalTick >= privacyStart
      && state.flames.length > 0)).toBe(true)
    await Promise.all(games.map((game) => expect.poll(() => latestSnapshot(game)!.state.bombs.length + latestSnapshot(game)!.state.flames.length).toBe(0)))
    const [hiddenCleanPng, sightedCleanPng] = await Promise.all([
      captureTilePng(one, 3, 1), captureTilePng(two, 3, 1),
    ])
    const secondEmptyFrame = latestSnapshot(games[1])!
    expect(secondEmptyFrame.state.arena.terrain[11 * 15 + 12]).toBe('floor')
    expect(secondEmptyFrame.state.bombs).toEqual([])
    expect(secondEmptyFrame.state.flames).toEqual([])
    const secondEmptyActor = visibleActor(secondEmptyFrame, 'two')!
    expect(Math.floor(secondEmptyActor.position.x / 1024)).toBe(13)
    expect(Math.floor(secondEmptyActor.position.y / 1024)).toBe(10)
    const emptySecondPng = await captureTilePng(two, 12, 11)
    const [hiddenPixels, sightedPixels, hiddenClean, sightedClean] = await Promise.all([
      decodeTilePng(one, reconnect.hiddenPng), decodeTilePng(two, reconnect.sightedPng),
      decodeTilePng(one, hiddenCleanPng), decodeTilePng(two, sightedCleanPng),
    ])
    expect(hiddenPixels).toEqual(hiddenClean)
    expect(sightedPixels).not.toEqual(sightedClean)
    expect(bombEvents(games[0], privacyStart).filter(({ tick }) => tick < deadline)).toHaveLength(0)
    expect(bombEvents(games[1], privacyStart).some(({ tick, kind }) => tick < deadline
      && kind.type === 'bombDetonated' && (kind.bomb.id === ownBomb.id
        || kind.bomb.id === rivalBomb.id))).toBe(true)
    await expect.poll(() => latestSnapshot(games[1])!.state.pickups).toContainEqual({
      cell: { column: 11, row: 11 }, kind: 'invisibleBombs' })
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 11, row: 11 })
    await Promise.all(games.map((game) => expect.poll(() => effectTicks(game, 'two')).toBeGreaterThan(0)))
    const secondCollection = snapshots(games[1]).flatMap(({ events }) => events).find(({ kind }) =>
      kind.type === 'pickupCollected' && kind.seat === 'two' && kind.kind === 'invisibleBombs')!
    const secondDeadline = secondCollection.tick + 600
    const observedDeadlines = games.map((game) => {
      const snapshot = latestSnapshot(game)!
      return snapshot.state.physicalTick + effectTicks(game, 'two')
    })
    expect(observedDeadlines).toEqual([secondDeadline, secondDeadline])
    const secondPrivacyStart = latestSnapshot(games[1])!.state.physicalTick
    await moveSeatTo(two, games[1], 'two', { column: 12, row: 11 })
    await expect.poll(() => latestSnapshot(games[0])!.state.physicalTick, { intervals: [15] })
      .toBeGreaterThanOrEqual(secondDeadline - 120)
    const survivor = await plant(two, games[1], games[0], { column: 12, row: 11 })
    expect(survivor.placedAt).toBeGreaterThanOrEqual(secondDeadline - 120)
    expect(survivor.placedAt).toBeLessThanOrEqual(secondDeadline - 60)
    expect(latestSnapshot(games[0])!.state.physicalTick).toBeLessThanOrEqual(secondDeadline - 60)
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 11 })
    await moveSeatTo(two, games[1], 'two', { column: 13, row: 10 })
    await expect.poll(() => latestSnapshot(games[1])!.state.physicalTick).toBeGreaterThanOrEqual(secondDeadline)
    const before = snapshots(games[1]).filter(({ state }) => state.physicalTick < secondDeadline)
      .findLast(({ state }) => snapshots(games[0]).some((frame) =>
        frame.state.physicalTick === state.physicalTick))!
    const beforeSighted = snapshots(games[0]).find(({ state }) =>
      state.physicalTick === before.state.physicalTick)!
    expect(before.state.physicalTick).toBeGreaterThanOrEqual(secondDeadline - 3)
    expect(before.state.bombs.some(({ id }) => id === survivor.id)).toBe(false)
    expect(beforeSighted.state.bombs.some(({ id }) => id === survivor.id)).toBe(true)
    await expect.poll(() => snapshots(games[1]).find(({ state }) => state.physicalTick >= secondDeadline
      && snapshots(games[0]).some((frame) => frame.state.physicalTick === state.physicalTick))
      ?.state.bombs.find(({ id }) => id === survivor.id)).toBeTruthy()
    const after = snapshots(games[1]).find(({ state }) => state.physicalTick >= secondDeadline
      && snapshots(games[0]).some((frame) => frame.state.physicalTick === state.physicalTick))!
    const afterSighted = snapshots(games[0]).find(({ state }) =>
      state.physicalTick === after.state.physicalTick)!
    expect(after.state.physicalTick).toBeLessThanOrEqual(secondDeadline + 2)
    const revealed = after.state.bombs.find(({ id }) => id === survivor.id)!
    const sighted = afterSighted.state.bombs.find(({ id }) => id === survivor.id)!
    expect(revealed.cell).toEqual({ column: 12, row: 11 })
    expect(revealed.trigger.type).toBe('timed')
    expect(sighted.trigger.type).toBe('timed')
    if (revealed.trigger.type !== 'timed' || sighted.trigger.type !== 'timed') {
      throw new Error('surviving fixture bomb must remain Timed')
    }
    expect(revealed.trigger.remainingTicks).toBe(sighted.trigger.remainingTicks)
    expect(bombEvents(games[0], secondPrivacyStart).filter(({ kind }) =>
      kind.type === 'bombPlaced' && kind.bomb.id === survivor.id)).toHaveLength(1)
    expect(bombEvents(games[1], secondPrivacyStart)).toHaveLength(0)
    const revealCaptureStart = expectRevealedPixelWindow(
      games[1], survivor.id, secondDeadline, after.room.roundId)
    const revealSecondPng = await captureTilePng(two, 12, 11)
    const revealCaptureEnd = expectRevealedPixelWindow(
      games[1], survivor.id, secondDeadline, after.room.roundId)
    const [emptySecondPixels, revealSecondPixels] = await Promise.all([
      decodeTilePng(two, emptySecondPng), decodeTilePng(two, revealSecondPng),
    ])
    expect(revealSecondPixels).not.toEqual(emptySecondPixels)
    info.annotations.push({ type: 'second-window-capture-ticks', description: JSON.stringify({
      collection: secondCollection.tick, placement: survivor.placedAt,
      captures: [revealCaptureStart, revealCaptureEnd],
    }) })
  } finally { await Promise.all(contexts.map((context) => context.close())) }
})
