import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { ArenaControls } from './controls'
import type { ArenaRenderer } from './renderer'
import type { GameArenaProps } from './types'
import './arena.css'

interface RendererOwner {
  host: HTMLElement
  preference: MediaQueryList
  disposed: boolean
  pending: Promise<void> | null
  renderer: ArenaRenderer | null
  attemptedActivation: number
}

interface RendererLifecycle {
  owner: { current: RendererOwner | null }
  latest: { current: GameArenaProps['snapshot'] }
  activation: { current: number }
  setFailed: (failed: boolean) => void
}

function startActivation(lifecycle: RendererLifecycle, current: RendererOwner, token: number): void {
  if (current.disposed || current.renderer || current.pending
    || current.attemptedActivation === token) return
  current.attemptedActivation = token
  lifecycle.setFailed(false)
  current.pending = import('./renderer').then(({ createArenaRenderer }) => {
    if (current.disposed || lifecycle.owner.current !== current) return null
    return createArenaRenderer(current.host)
  }).then((created) => {
    current.pending = null
    if (!created) return
    if (current.disposed || lifecycle.owner.current !== current) { created.destroy(); return }
    try {
      created.setReducedMotion(current.preference.matches)
      if (lifecycle.latest.current) created.update(lifecycle.latest.current)
      else created.suspend()
    } catch (error) {
      try { created.destroy() } catch { /* Retain the initialization error. */ }
      throw error
    }
    current.renderer = created
    lifecycle.setFailed(false)
  }).catch(() => {
    current.pending = null
    if (current.disposed || lifecycle.owner.current !== current) return
    if (lifecycle.latest.current && lifecycle.activation.current !== token) {
      startActivation(lifecycle, current, lifecycle.activation.current)
    } else {
      lifecycle.setFailed(true)
    }
  })
}

function applyOwnedRenderer(
  lifecycle: RendererLifecycle, current: RendererOwner,
  operation: (renderer: ArenaRenderer) => void,
): void {
  const renderer = current.renderer
  if (!renderer) return
  try {
    operation(renderer)
  } catch {
    current.renderer = null
    current.attemptedActivation = lifecycle.activation.current
    try { renderer.destroy() } catch { /* Retain the operation failure. */ }
    lifecycle.setFailed(true)
  }
}

export function GameArena(props: GameArenaProps) {
  const { snapshot, enabled, bindings, onDirection, onBomb, onEquipment } = props
  const host = useRef<HTMLDivElement>(null)
  const owner = useRef<RendererOwner | null>(null)
  const latest = useRef(snapshot)
  const activation = useRef(snapshot ? 1 : 0)
  const hadSnapshot = useRef(snapshot !== null)
  const previousIdentity = useRef('')
  const [failed, setFailed] = useState(false)
  const [rendererLifecycle] = useState<RendererLifecycle>(() => ({ owner, latest, activation, setFailed }))
  const [controls] = useState(() => new ArenaControls({
    enabled, bindings, onDirection, onBomb, onEquipment,
  }))
  const hasSnapshot = snapshot !== null
  const active = enabled && !failed

  useLayoutEffect(() => {
    controls.configure({ enabled: active, bindings, onDirection, onBomb, onEquipment })
  }, [active, bindings, onDirection, onBomb, onEquipment, controls])

  useLayoutEffect(() => {
    latest.current = snapshot
    const hasCurrent = snapshot !== null
    if (hasCurrent && !hadSnapshot.current) activation.current += 1
    hadSnapshot.current = hasCurrent
    const identity = snapshot ? `${snapshot.room.roomId}:${snapshot.room.roundId}` : ''
    if (identity !== previousIdentity.current) {
      controls.setFocused(false)
      previousIdentity.current = identity
    }
    const current = owner.current
    if (!current) return
    if (snapshot) {
      if (current.renderer) {
        applyOwnedRenderer(rendererLifecycle, current, (renderer) => renderer.update(snapshot))
      }
      else startActivation(rendererLifecycle, current, activation.current)
    } else {
      applyOwnedRenderer(rendererLifecycle, current, (renderer) => renderer.suspend())
    }
  }, [snapshot, controls, rendererLifecycle])

  useEffect(() => {
    const blur = (): void => controls.setFocused(false)
    const visibility = (): void => { if (document.hidden) blur() }
    const release = (event: KeyboardEvent): void => controls.releasePhysicalKey(event.code)
    const interval = window.setInterval(() => controls.refresh(), 100)
    window.addEventListener('blur', blur)
    window.addEventListener('keyup', release)
    document.addEventListener('visibilitychange', visibility)
    return () => {
      window.clearInterval(interval)
      window.removeEventListener('blur', blur)
      window.removeEventListener('keyup', release)
      document.removeEventListener('visibilitychange', visibility)
      controls.clear()
    }
  }, [controls])

  useLayoutEffect(() => {
    if (!host.current) return
    const preference = window.matchMedia('(prefers-reduced-motion: reduce)')
    const current: RendererOwner = {
      host: host.current, preference, disposed: false, pending: null, renderer: null,
      attemptedActivation: 0,
    }
    owner.current = current
    const motion = (): void => current.renderer?.setReducedMotion(preference.matches)
    preference.addEventListener('change', motion)
    if (latest.current) startActivation(rendererLifecycle, current, activation.current)
    return () => {
      current.disposed = true
      preference.removeEventListener('change', motion)
      current.renderer?.destroy()
      current.renderer = null
      if (owner.current === current) owner.current = null
    }
  }, [rendererLifecycle])

  return (
    <div
      className="game-arena"
      style={snapshot ? { aspectRatio: `${snapshot.state.arena.width} / ${snapshot.state.arena.height}` } : undefined}
      role="region"
      aria-label={props.label}
      aria-describedby={props.instructionsId}
      tabIndex={active ? 0 : -1}
      onFocus={() => controls.setFocused(true)}
      onBlur={() => controls.setFocused(false)}
      onPointerDown={(event) => {
        if (active) { event.currentTarget.focus(); controls.setFocused(true) }
      }}
      onKeyDown={(event) => {
        if (event.target === event.currentTarget) {
          controls.setFocused(true)
          controls.keyDown(event.nativeEvent)
        }
      }}
      onKeyUp={(event) => { if (event.target === event.currentTarget) controls.keyUp(event.nativeEvent) }}
    >
      <div className="game-arena__canvas" ref={host} />
      {!hasSnapshot && <p className="game-arena__message">{props.waitingText}</p>}
      {hasSnapshot && failed && <p role="alert" className="game-arena__message">{props.failureText}</p>}
      {hasSnapshot && active && <span className="game-arena__focus">{props.focusHint}</span>}
    </div>
  )
}
