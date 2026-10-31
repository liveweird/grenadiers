import type { GameClientState } from './clientTypes'
import { format, useGameCopy, type GameCopy } from './locale'
import { projectCpuDebug } from './cpuDebugProjection'
import type { AvailableCpuDebugRow } from './cpuDebugTypes'
import type { CpuDebugState } from './useCpuDebug'
import type { PickupKind } from '../v17/schema'
import { useState } from 'react'

function cell(copy: GameCopy, value: { column: number; row: number }): string {
  return format(copy.cpuDebug.cell, value)
}

function pickupName(copy: GameCopy, kind: PickupKind): string {
  if (kind === 'bombCapacity' || kind === 'blastRange' || kind === 'movementSpeed') return copy.pickups[kind]
  return copy.items[kind]
}

function targetText(copy: GameCopy, target: AvailableCpuDebugRow['target']): string {
  if (!target) return copy.cpuDebug.noTarget
  if (target.type === 'pickup') {
    return format(copy.cpuDebug.target.pickup, { item: pickupName(copy, target.kind), cell: cell(copy, target.cell) })
  }
  if (target.type === 'block') return format(copy.cpuDebug.target.block, { cell: cell(copy, target.cell) })
  if (target.type === 'opponent') {
    return format(copy.cpuDebug.target.opponent, { player: copy.player[target.seat] })
  }
  return format(copy.cpuDebug.target.tile, { cell: cell(copy, target.cell) })
}

function IntentDetails({ row, copy }: { row: AvailableCpuDebugRow; copy: GameCopy }) {
  const facts = [
    [copy.cpuDebug.objectiveLabel, copy.cpuDebug.objective[row.objective]],
    [copy.cpuDebug.targetLabel, targetText(copy, row.target)],
    [copy.cpuDebug.approachLabel, row.approach ? cell(copy, row.approach) : copy.cpuDebug.noApproach],
    [copy.cpuDebug.phaseLabel, copy.cpuDebug.phase[row.phase]],
    [copy.cpuDebug.switchReasonLabel, copy.cpuDebug.switchReason[row.switchReason]],
    [copy.cpuDebug.observedTickLabel, String(row.observedTick)],
  ]
  return <dl className="cpu-debug-facts">{facts.map(([label, value]) => <div key={label}>
    <dt>{label}</dt><dd>{value}</dd>
  </div>)}</dl>
}

export function CpuDebugPanel({ state, debug }: {
  state: GameClientState
  debug: CpuDebugState
}) {
  const copy = useGameCopy()
  const [open, setOpen] = useState(false)
  const rows = projectCpuDebug(debug.type === 'ready' ? debug.response : null, state)
  return <section className="cpu-debug-panel" aria-label={copy.cpuDebug.title}>
    <button type="button" className="cpu-debug-toggle" aria-label={copy.cpuDebug.title} aria-expanded={open}
      aria-controls="cpu-debug-content" onClick={() => setOpen((current) => !current)}>
      {copy.cpuDebug.title}
    </button>
    {open && <div id="cpu-debug-content">
      <p className="cpu-debug-note">{copy.cpuDebug.developerOnly}</p>
      {debug.type === 'disabled' && <p role="status">{copy.cpuDebug.disabled}</p>}
      {debug.type === 'error' && <p role="alert">{copy.cpuDebug.error}</p>}
      {debug.type === 'waiting' && <p role="status">{copy.cpuDebug.waiting}</p>}
      {rows.length === 0 && debug.type === 'ready' && <p>{copy.cpuDebug.noCpus}</p>}
      {rows.map(({ seat, intent }) => <section className="cpu-debug-row" key={seat}
        aria-label={format(copy.cpuDebug.cpuLabel, { player: copy.player[seat] })}>
        <h3>{format(copy.cpuDebug.cpuLabel, { player: copy.player[seat] })}</h3>
        {intent?.status === 'available'
          ? <IntentDetails row={intent} copy={copy} />
          : <p className="cpu-debug-unavailable">{copy.cpuDebug.unavailable}</p>}
      </section>)}
    </div>}
  </section>
}
