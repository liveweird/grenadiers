import { Button, Text } from '@mantine/core'
import { useEffect, useRef, useState } from 'react'
import type { GameClientState } from '../v6/client'
import type { V6Copy } from '../v6/locale'
import { AudioCursor } from './cursor'
import { GameAudio } from './engine'
import { loadAudioPreference, saveAudioPreference } from './preferences'

export function SoundControl({ state, copy }: { state: GameClientState; copy: V6Copy }) {
  const [preferred, setPreferred] = useState(loadAudioPreference)
  const [engineRunning, setEngineRunning] = useState(false)
  const [visible, setVisible] = useState(() => document.visibilityState === 'visible')
  const [failed, setFailed] = useState(false)
  const [cursor] = useState(() => new AudioCursor())
  const audio = useRef<GameAudio | null>(null)
  const latest = useRef(state)
  const previousRoom = useRef(state.room?.roomId ?? null)
  const playing = preferred && visible && engineRunning

  useEffect(() => { latest.current = state }, [state])

  useEffect(() => {
    const engine = new GameAudio(undefined, () => {
      if (audio.current === engine) setEngineRunning(engine.isEnabled())
    })
    audio.current = engine
    return () => {
      if (audio.current === engine) audio.current = null
      engine.dispose()
    }
  }, [])

  useEffect(() => {
    const visibility = (): void => {
      const nextVisible = document.visibilityState === 'visible'
      setVisible(nextVisible)
      cursor.consume(latest.current, false)
      if (!nextVisible) audio.current?.silence()
    }
    document.addEventListener('visibilitychange', visibility)
    return () => document.removeEventListener('visibilitychange', visibility)
  }, [cursor])

  useEffect(() => {
    const roomId = state.room?.roomId ?? null
    const lostRoom = previousRoom.current !== null && roomId === null
    previousRoom.current = roomId
    const engine = audio.current
    if (lostRoom || (state.room && state.connection !== 'authenticated')) engine?.silence()
    const audible = visible && state.connection === 'authenticated' && Boolean(engine?.isEnabled())
    engine?.play(cursor.consume(state, audible))
  }, [cursor, state, visible])

  useEffect(() => {
    const dispose = (): void => audio.current?.dispose()
    window.addEventListener('pagehide', dispose)
    return () => window.removeEventListener('pagehide', dispose)
  }, [])

  const toggle = async (): Promise<void> => {
    const engine = audio.current
    if (!engine) return
    if (engine.isEnabled()) {
      saveAudioPreference(false)
      setPreferred(false)
      setFailed(false)
      engine.silence()
      return
    }
    saveAudioPreference(true)
    setPreferred(true)
    setFailed(false)
    cursor.consume(latest.current, false)
    const enabled = await engine.enableFromGesture()
    if (audio.current !== engine) return
    if (enabled) cursor.consume(latest.current, true)
    else setFailed(true)
  }

  const status = failed ? copy.audio.unavailable : playing ? copy.audio.on
    : preferred ? copy.audio.paused : copy.audio.off
  const label = playing ? copy.audio.mute : preferred ? copy.audio.resume : copy.audio.enable
  return (
    <div className="sound-control">
      <Button variant="subtle" color="dark" size="compact-sm" aria-label={label}
        onClick={() => { void toggle() }}>{label}</Button>
      <Text component="span" size="xs" aria-live="polite">{status}</Text>
    </div>
  )
}
