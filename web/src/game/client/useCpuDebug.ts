import { useEffect, useRef, useState } from 'react'
import validateCpuDebugResponse from '../../validation/generated/cpu-debug.mjs'
import type { GameClient } from './client'
import type { GameClientState } from './clientTypes'
import { cpuDebugResponseMatchesState } from './cpuDebugProjection'
import type { CpuDebugResponse } from './cpuDebugTypes'

export type CpuDebugState =
  | { type: 'hidden' | 'waiting' | 'disabled' | 'error' }
  | { type: 'ready'; response: CpuDebugResponse }

function eligible(state: GameClientState): boolean {
  const own = state.room?.seats.find(({ seat }) => seat === state.youSeat)
  return state.connection === 'authenticated' && !state.admissionPending && !state.leavePending
    && own?.controller.type === 'human' && own.connection.type === 'connected'
}

export function useCpuDebug(state: GameClientState, client: GameClient, enabled: boolean): CpuDebugState {
  const [stored, setStored] = useState<{
    key: string
    scope: GameClientState['admissionBaseline']
    debug: CpuDebugState
  } | null>(null)
  const [serverDisabled, setServerDisabled] = useState(false)
  const latest = useRef(state)
  const lastRequestStartedAt = useRef(Number.NEGATIVE_INFINITY)
  useEffect(() => { latest.current = state }, [state])
  const roomId = state.room?.roomId ?? null
  const roundId = state.room?.roundId ?? null
  const youSeat = state.youSeat
  const active = enabled && eligible(state) && roomId !== null && roundId !== null && youSeat !== null
  const key = `${roomId}/${roundId}/${youSeat}`
  const scope = state.admissionBaseline

  useEffect(() => {
    if (!active || serverDisabled) return
    let live = true
    let timer: ReturnType<typeof setTimeout> | null = null
    let acceptedTick = -1
    let acceptedRevision = -1
    const controller = new AbortController()
    const schedulePoll = (poll: () => Promise<void>): void => {
      const delay = Math.max(0, 500 - (performance.now() - lastRequestStartedAt.current))
      if (delay === 0) void poll()
      else timer = setTimeout(() => { void poll() }, delay)
    }
    const poll = async (): Promise<void> => {
      lastRequestStartedAt.current = performance.now()
      try {
        const result = await client.requestCpuDebug(controller.signal)
        if (!live || result.type !== 'response') return
        if (result.response.status === 404) {
          setServerDisabled(true)
          return
        }
        if (!result.response.ok) throw new Error('CPU diagnostics request failed')
        const payload: unknown = await result.response.json()
        if (!validateCpuDebugResponse(payload)) throw new Error('Invalid CPU diagnostics response')
        const baseline = result.baseline
        if (payload.roomId !== baseline.roomId || payload.roundId !== baseline.roundId
          || payload.youSeat !== baseline.youSeat || payload.tick < baseline.tick
          || payload.revision < baseline.revision || payload.tick < acceptedTick
          || payload.revision < acceptedRevision || !cpuDebugResponseMatchesState(payload, latest.current)) {
          throw new Error('Stale CPU diagnostics response')
        }
        acceptedTick = payload.tick
        acceptedRevision = payload.revision
        setStored({ key, scope, debug: { type: 'ready', response: payload } })
      } catch {
        if (live && !controller.signal.aborted) setStored({ key, scope, debug: { type: 'error' } })
      }
      if (live && !controller.signal.aborted) schedulePoll(poll)
    }
    schedulePoll(poll)
    return () => {
      live = false
      controller.abort()
      if (timer) clearTimeout(timer)
    }
  }, [active, client, key, scope, serverDisabled])

  if (!enabled) return { type: 'hidden' }
  if (serverDisabled) return { type: 'disabled' }
  if (!active || stored?.key !== key || stored.scope !== scope) return { type: 'waiting' }
  return stored.debug
}
