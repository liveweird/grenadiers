import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { ArenaControls } from './controls'
import type { ArenaRenderer } from './renderer'
import type { GameArenaProps } from './types'
import './arena.css'

export function GameArena(props: GameArenaProps) {
  const { snapshot, enabled, bindings, onDirection, onBomb } = props
  const host = useRef<HTMLDivElement>(null)
  const renderer = useRef<ArenaRenderer | null>(null)
  const latest = useRef(snapshot)
  const previousIdentity = useRef('')
  const [failed, setFailed] = useState(false)
  const [controls] = useState(() => new ArenaControls({ enabled, bindings, onDirection, onBomb }))
  const hasSnapshot = snapshot !== null
  const active = enabled && !failed

  useLayoutEffect(() => {
    controls.configure({ enabled: active, bindings, onDirection, onBomb })
  }, [active, bindings, onDirection, onBomb, controls])

  useLayoutEffect(() => {
    latest.current = snapshot
    const identity = snapshot ? `${snapshot.room.roomId}:${snapshot.room.roundId}` : ''
    if (identity !== previousIdentity.current) {
      controls.setFocused(false)
      previousIdentity.current = identity
    }
    if (snapshot) renderer.current?.update(snapshot)
  }, [snapshot, controls])

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

  useEffect(() => {
    if (!hasSnapshot || !host.current) return
    const element = host.current
    const preference = window.matchMedia('(prefers-reduced-motion: reduce)')
    const motion = (): void => renderer.current?.setReducedMotion(preference.matches)
    let disposed = false
    let instance: ArenaRenderer | null = null
    preference.addEventListener('change', motion)
    void import('./renderer').then(({ createArenaRenderer }) => createArenaRenderer(element))
      .then((created) => {
        if (disposed) { created.destroy(); return }
        instance = created
        renderer.current = created
        motion()
        if (latest.current) created.update(latest.current)
        setFailed(false)
      })
      .catch(() => { if (!disposed) setFailed(true) })
    return () => {
      disposed = true
      preference.removeEventListener('change', motion)
      instance?.destroy()
      if (renderer.current === instance) renderer.current = null
    }
  }, [hasSnapshot])

  return (
    <div
      className="game-arena"
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
