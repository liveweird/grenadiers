import { Button, Text } from '@mantine/core'
import { useEffect, useId, useRef, useState } from 'react'
import type { GameClientState } from '../client/client'
import type { GameCopy } from '../client/locale'
import type { PresentationDeaths } from '../arena/types'
import { AudioCursor } from './cursor'
import { GameAudio } from './engine'
import { loadAudioPreference, saveAudioPreference } from './preferences'

export function SoundControl({ state, copy, presentationDeaths }: {
  state: GameClientState; copy: GameCopy; presentationDeaths?: PresentationDeaths | undefined
}) {
  const [preference, setPreference] = useState(loadAudioPreference)
  const [engineRunning, setEngineRunning] = useState(false)
  const [visible, setVisible] = useState(() => document.visibilityState === 'visible')
  const [failed, setFailed] = useState(false)
  const [cursor] = useState(() => new AudioCursor())
  const initialVolume = useRef(preference.volume)
  const audio = useRef<GameAudio | null>(null)
  const latest = useRef(state)
  const latestDeaths = useRef(presentationDeaths)
  const previousRoom = useRef(state.room?.roomId ?? null)
  const volumeId = useId()
  const playing = preference.enabled && visible && engineRunning

  useEffect(() => {
    latest.current = state
    latestDeaths.current = presentationDeaths
  }, [presentationDeaths, state])

  useEffect(() => {
    const engine = new GameAudio(undefined, () => {
      if (audio.current === engine) setEngineRunning(engine.isEnabled())
    })
    engine.setVolume(initialVolume.current)
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
      cursor.consume(latest.current, false, latestDeaths.current)
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
    engine?.play(cursor.consume(state, audible, presentationDeaths))
  }, [cursor, presentationDeaths, state, visible])

  useEffect(() => {
    const pause = (): void => audio.current?.silence()
    window.addEventListener('pagehide', pause)
    return () => window.removeEventListener('pagehide', pause)
  }, [])

  const toggle = async (): Promise<void> => {
    const engine = audio.current
    if (!engine) return
    if (engine.isEnabled()) {
      const next = { ...preference, enabled: false }
      saveAudioPreference(next)
      setPreference(next)
      setFailed(false)
      engine.silence()
      return
    }
    const next = { ...preference, enabled: true }
    saveAudioPreference(next)
    setPreference(next)
    setFailed(false)
    cursor.consume(latest.current, false, latestDeaths.current)
    const enabled = await engine.enableFromGesture()
    if (audio.current !== engine) return
    if (enabled) cursor.consume(latest.current, true, latestDeaths.current)
    else setFailed(true)
  }

  const changeVolume = (value: number): void => {
    const next = { ...preference, volume: value }
    setPreference(next)
    saveAudioPreference(next)
    audio.current?.setVolume(value)
  }

  const status = failed ? copy.audio.unavailable : playing ? copy.audio.on
    : preference.enabled ? copy.audio.paused : copy.audio.off
  const label = playing ? copy.audio.mute : preference.enabled ? copy.audio.resume : copy.audio.enable
  return (
    <div className="sound-control">
      <Button variant="subtle" color="dark" size="compact-sm" aria-label={label}
        onClick={() => { void toggle() }}>{label}</Button>
      <Text component="span" size="xs" aria-live="polite">{status}</Text>
      <label htmlFor={volumeId}>{copy.audio.volume}</label>
      <input id={volumeId} type="range" min="0" max="100" step="5" value={preference.volume}
        aria-label={copy.audio.volume} aria-valuetext={`${preference.volume}%`} onChange={(event) => changeVolume(Number(event.currentTarget.value))} />
      <span aria-hidden="true">{preference.volume}%</span>
    </div>
  )
}
