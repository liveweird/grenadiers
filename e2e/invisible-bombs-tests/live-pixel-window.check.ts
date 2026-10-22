import assert from 'node:assert/strict'
import { requireSuccessfulTasks, settleAll, validateLiveGuard,
  type LiveWindowDiagnostics } from './live-pixel-window-control.ts'

const timed = { id: 7, trigger: { type: 'timed', remainingTicks: 40 } }
const valid = { currentRoundId: 3, expectedRoundId: 3, physicalTick: 100,
  effectTicks: 500, effectDeadline: 600, bombId: 7, bomb: timed }
validateLiveGuard(valid)
for (const changed of [
  { ...valid, bomb: undefined },
  { ...valid, bomb: { ...timed, trigger: { type: 'timed', remainingTicks: 0 } } },
  { ...valid, effectDeadline: 601 },
  { ...valid, currentRoundId: 4 },
]) assert.throws(() => validateLiveGuard(changed))

const diagnostics: LiveWindowDiagnostics = { stages: [], selectedBombs: [], events: [] }
const retreatFailure = new Error('retreat failed')
assert.throws(() => requireSuccessfulTasks(diagnostics, [
  { name: 'capture', ok: true, value: undefined },
  { name: 'one-retreat', ok: false, error: retreatFailure },
]), (error) => error instanceof AggregateError && error.errors[0] === retreatFailure)
assert.equal(diagnostics.stages.at(-1)?.stage, 'live-window-phase')
assert.equal(diagnostics.stages.at(-1)?.outcome, 'failed')

const captureFailure = new Error('capture failed')
let releasePeer!: () => void
let peerDrained = false
const peerCapture = new Promise<void>((resolve) => { releasePeer = resolve }).then(() => {
  peerDrained = true
  return Buffer.from('peer')
})
const drainingCapture = settleAll([Promise.reject(captureFailure), peerCapture])
let phaseSettled = false
void drainingCapture.then(() => { phaseSettled = true }, () => { phaseSettled = true })
await Promise.resolve()
assert.equal(phaseSettled, false)
releasePeer()
await assert.rejects(drainingCapture, (error) =>
  error instanceof AggregateError && error.errors[0] === captureFailure)
assert.equal(peerDrained, true)
console.log('live pixel window checks: PASS')
