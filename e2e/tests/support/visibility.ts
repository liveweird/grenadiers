import { spawn, type ChildProcess } from 'node:child_process'
import { mkdtemp, rm } from 'node:fs/promises'
import { tmpdir } from 'node:os'
import { join } from 'node:path'
import { setTimeout as delay } from 'node:timers/promises'
import { chromium, expect, type Browser, type BrowserContext, type Page } from '@playwright/test'
import type { GameServerMessage, Position, Seat } from '../../../web/src/game/v12/schema'
import { visibleActor, latestSnapshot, type ObservedGame } from './duel'
import {
  publicErrorSummary, runSanitizedCleanup, sanitizePublicError, sanitizeStartupText,
  VisibilityStartupError, waitForChromiumEndpoint,
  type VisibilityStartupDiagnostics,
} from './visibility-startup'

type Snapshot = Extract<GameServerMessage, { type: 'snapshot' }>
type VisibilityWindow = Window & { grenadiersVisibilityChanges?: DocumentVisibilityState[] }
type OwnedProcess = { child: ChildProcess; exited: Promise<void> }
const SHUTDOWN_GRACE_MS = 2_000

function ownProcess(child: ChildProcess): OwnedProcess {
  const exited = new Promise<void>((resolve) => {
    if (child.exitCode !== null || child.signalCode !== null) resolve()
    else {
      child.once('error', () => resolve())
      child.once('exit', () => resolve())
    }
  })
  return { child, exited }
}

async function exitedWithin(owned: OwnedProcess, timeout: number): Promise<boolean> {
  return Promise.race([
    owned.exited.then(() => true),
    delay(timeout, false, { ref: false }),
  ])
}

function signalOwnedProcess(owned: OwnedProcess, signal: NodeJS.Signals): void {
  if (owned.child.exitCode !== null || owned.child.signalCode !== null) return
  try {
    if (owned.child.pid && process.platform !== 'win32') process.kill(-owned.child.pid, signal)
    else owned.child.kill(signal)
  } catch (error) {
    if ((error as NodeJS.ErrnoException).code !== 'ESRCH') throw error
  }
}

async function terminateOwnedProcess(owned: OwnedProcess, browser?: Browser): Promise<void> {
  if (browser) {
    const closed = browser.close().then(() => true, () => true)
    if (await Promise.race([closed, delay(SHUTDOWN_GRACE_MS, false, { ref: false })])
      && await exitedWithin(owned, 0)) return
  }
  signalOwnedProcess(owned, 'SIGTERM')
  if (await exitedWithin(owned, SHUTDOWN_GRACE_MS)) return
  signalOwnedProcess(owned, 'SIGKILL')
  if (await exitedWithin(owned, SHUTDOWN_GRACE_MS)) return
  throw new Error('Owned Chromium process did not exit after SIGKILL')
}

async function cleanupOwnedBrowser(owned: OwnedProcess, profile: string, browser?: Browser): Promise<void> {
  await terminateOwnedProcess(owned, browser)
  await rm(profile, { recursive: true, force: true })
}

export async function launchNaturalVisibilityBrowser(): Promise<{
  browser: Browser
  context: BrowserContext
  startupDiagnostics: () => VisibilityStartupDiagnostics
  close: () => Promise<void>
}> {
  const profile = await mkdtemp(join(tmpdir(), 'grenadiers-visibility-'))
  const child = spawn(chromium.executablePath(), [
    '--no-sandbox', '--disable-dev-shm-usage', '--no-first-run', '--no-default-browser-check',
    '--enable-gpu', '--use-gl=angle', '--use-angle=swiftshader',
    '--remote-debugging-port=0', `--user-data-dir=${profile}`, 'about:blank',
  ], { detached: process.platform !== 'win32', stdio: ['ignore', 'ignore', 'pipe'] })
  const owned = ownProcess(child)
  const startup = waitForChromiumEndpoint(child, profile, 10_000)
  let browser!: Browser
  let connectionError: unknown
  try {
    browser = await chromium.connectOverCDP(await startup.endpoint, { noDefaults: true })
  } catch (error) {
    connectionError = error
  }
  if (connectionError !== undefined) {
    const error = sanitizePublicError(connectionError, profile)
    let cleanupError: unknown
    try {
      await cleanupOwnedBrowser(owned, profile)
    } catch (caught) {
      cleanupError = caught
    } finally {
      startup.dispose()
    }
    const startupError = new VisibilityStartupError(
      error instanceof Error ? sanitizeStartupText(error.message, profile) : 'Chromium startup failed',
      startup.diagnostics(), { cause: error },
    )
    if (cleanupError !== undefined) {
      const safeCleanup = sanitizePublicError(cleanupError, profile)
      throw new AggregateError([startupError, safeCleanup],
        `Chromium startup failed (${publicErrorSummary(error)}); owned-process cleanup failed `
        + `(${publicErrorSummary(safeCleanup)})`, { cause: error })
    }
    throw startupError
  }
  const context = browser.contexts()[0]
  if (!context) {
    let cleanupError: unknown
    try {
      await cleanupOwnedBrowser(owned, profile, browser)
    } catch (caught) {
      cleanupError = caught
    } finally {
      startup.dispose()
    }
    const unavailable = new VisibilityStartupError('Chromium default context is unavailable',
      startup.diagnostics())
    if (cleanupError !== undefined) {
      const safeCleanup = sanitizePublicError(cleanupError, profile)
      throw new AggregateError([unavailable, safeCleanup],
        `Chromium context lookup failed (${publicErrorSummary(unavailable)}); `
        + `owned-process cleanup failed (${publicErrorSummary(safeCleanup)})`,
        { cause: unavailable })
    }
    throw unavailable
  }
  let closing: Promise<void> | undefined
  return {
    browser,
    context,
    startupDiagnostics: startup.diagnostics,
    async close() {
      closing ??= runSanitizedCleanup(
        () => cleanupOwnedBrowser(owned, profile, browser), profile).finally(startup.dispose)
      await closing
    },
  }
}

function actorPosition(game: ObservedGame, seat: Seat): Position {
  const actor = visibleActor(latestSnapshot(game), seat)
  if (!actor?.alive) throw new Error(`Visibility recovery requires living seat ${seat}`)
  return actor.position
}

export async function expectVisibility(page: Page, expected: DocumentVisibilityState): Promise<void> {
  await expect.poll(() => page.evaluate(() => document.visibilityState), { intervals: [15] })
    .toBe(expected)
}

export async function observeVisibilityChanges(page: Page): Promise<void> {
  await page.evaluate(() => {
    const observed = window as VisibilityWindow
    observed.grenadiersVisibilityChanges = []
    document.addEventListener('visibilitychange', () => {
      observed.grenadiersVisibilityChanges?.push(document.visibilityState)
    })
  })
}

export async function visibilityChanges(page: Page): Promise<DocumentVisibilityState[]> {
  return page.evaluate(() => (window as VisibilityWindow).grenadiersVisibilityChanges ?? [])
}

export async function expectStableFreshPosition(
  game: ObservedGame,
  seat: Seat,
  afterRevision: number,
): Promise<{ position: Position; revision: number }> {
  await expect.poll(() => {
    const frames = game.frames.filter((frame): frame is Snapshot => frame.type === 'snapshot'
      && frame.room.revision > afterRevision).slice(-3)
    if (frames.length !== 3) return false
    const positions = frames.map((frame) => visibleActor(frame, seat)?.position)
    const first = positions[0]
    return first !== undefined && positions.every((position) => position !== undefined
      && position.x === first.x && position.y === first.y)
  }, { intervals: [15] }).toBe(true)
  const snapshot = latestSnapshot(game)!
  return { position: actorPosition(game, seat), revision: snapshot.room.revision }
}

export function position(game: ObservedGame, seat: Seat): Position {
  return actorPosition(game, seat)
}
