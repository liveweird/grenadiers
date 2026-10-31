import { act, renderHook } from '@testing-library/react'
import { afterEach, describe, expect, it, vi } from 'vitest'
import type { GameClient } from './client'
import type { GameClientState } from './clientTypes'
import type { CpuDebugHttpResult, CpuDebugResponse } from './cpuDebugTypes'
import { useCpuDebug } from './useCpuDebug'
import { snapshot } from './testSupport'

function state(): GameClientState {
  const current = snapshot(20)
  current.room.configuration.controllers[1] = 'cpu'
  current.room.configuration.cpuDifficulty = 'standard'
  current.room.seats[1].controller = { type: 'cpu', difficulty: 'standard' }
  return {
    connection: 'authenticated', room: current.room, snapshot: current, youSeat: 'one', failure: null,
    admissionPending: false, leavePending: false, settingsPending: false, reconnectAttempt: 0,
    admissionBaseline: {
      roomId: current.room.roomId, revision: 20, tick: 20, physicalTick: 20,
    },
  }
}

function payload(tick = 20, revision = tick): CpuDebugResponse {
  return {
    protocolVersion: 1, roomId: 'b'.repeat(32), roundId: 1, tick, revision, youSeat: 'one',
    cpus: [{ seat: 'two', status: 'unavailable' }],
  }
}

function result(value: CpuDebugResponse, baselineTick = 20): CpuDebugHttpResult {
  return {
    type: 'response', response: new Response(JSON.stringify(value), { status: 200 }),
    baseline: { roomId: 'b'.repeat(32), roundId: 1, tick: baselineTick, revision: baselineTick, youSeat: 'one' },
  }
}

function client(request: ReturnType<typeof vi.fn>): GameClient {
  return { requestCpuDebug: request } as unknown as GameClient
}

afterEach(() => { vi.useRealTimers() })

describe('CPU debug polling', () => {
  it('makes no request when the URL opt-in is absent', () => {
    const request = vi.fn()
    const gameClient = client(request)
    renderHook(() => useCpuDebug(state(), gameClient, false))
    expect(request).not.toHaveBeenCalled()
  })

  it('keeps one request in flight and starts no more than two per second', async () => {
    vi.useFakeTimers()
    let resolve!: (value: CpuDebugHttpResult) => void
    const first = new Promise<CpuDebugHttpResult>((done) => { resolve = done })
    const request = vi.fn().mockReturnValueOnce(first).mockResolvedValue(result(payload()))
    const gameClient = client(request)
    renderHook(() => useCpuDebug(state(), gameClient, true))
    expect(request).toHaveBeenCalledTimes(1)
    await act(async () => { await vi.advanceTimersByTimeAsync(1_000) })
    expect(request).toHaveBeenCalledTimes(1)
    await act(async () => { resolve(result(payload())); await first })
    await act(async () => { await vi.advanceTimersByTimeAsync(0) })
    expect(request).toHaveBeenCalledTimes(2)
    await act(async () => { await vi.advanceTimersByTimeAsync(499) })
    expect(request).toHaveBeenCalledTimes(2)
    await act(async () => { await vi.advanceTimersByTimeAsync(1) })
    expect(request).toHaveBeenCalledTimes(3)
  })

  it('accepts a response baseline even when the public game advances during the request', async () => {
    let resolve!: (value: CpuDebugHttpResult) => void
    const pending = new Promise<CpuDebugHttpResult>((done) => { resolve = done })
    const request = vi.fn().mockReturnValue(pending)
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current }) => useCpuDebug(current, gameClient, true), {
      initialProps: { current: initial },
    })
    const advanced = { ...initial, room: { ...initial.room!, tick: 22, revision: 22 } }
    hook.rerender({ current: advanced })
    await act(async () => { resolve(result(payload(21), 20)); await pending })
    expect(hook.result.current.type).toBe('ready')
  })

  it('does not restart an in-flight poll across rapid same-round snapshots', async () => {
    let resolve!: (value: CpuDebugHttpResult) => void
    const pending = new Promise<CpuDebugHttpResult>((done) => { resolve = done })
    const request = vi.fn().mockReturnValue(pending)
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current }) => useCpuDebug(current, gameClient, true), {
      initialProps: { current: initial },
    })
    for (let revision = 21; revision <= 40; revision += 1) {
      const current = snapshot(revision)
      current.room.configuration.controllers[1] = 'cpu'
      current.room.configuration.cpuDifficulty = 'standard'
      current.room.seats[1].controller = { type: 'cpu', difficulty: 'standard' }
      hook.rerender({ current: { ...initial, room: current.room, snapshot: current } })
    }
    expect(request).toHaveBeenCalledTimes(1)
    expect((request.mock.calls[0][0] as AbortSignal).aborted).toBe(false)
    await act(async () => { resolve(result(payload(21), 20)); await pending })
    expect(hook.result.current.type).toBe('ready')
  })

  it('aborts and clears displayed diagnostics on disconnect', async () => {
    const request = vi.fn().mockResolvedValue(result(payload()))
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current }) => useCpuDebug(current, gameClient, true), {
      initialProps: { current: initial },
    })
    await act(async () => { await Promise.resolve() })
    expect(hook.result.current.type).toBe('ready')
    const signal = request.mock.calls[0][0] as AbortSignal
    hook.rerender({ current: { ...initial, connection: 'disconnected' as const, room: null, snapshot: null } })
    expect(signal.aborted).toBe(true)
    expect(hook.result.current.type).toBe('waiting')
  })

  it('does not restore cached rows or exceed the start rate across a same-room reconnect', async () => {
    vi.useFakeTimers()
    let resolveFresh!: (value: CpuDebugHttpResult) => void
    const fresh = new Promise<CpuDebugHttpResult>((done) => { resolveFresh = done })
    const starts: number[] = []
    const request = vi.fn()
      .mockImplementationOnce(() => { starts.push(performance.now()); return Promise.resolve(result(payload())) })
      .mockImplementationOnce(() => { starts.push(performance.now()); return fresh })
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current }) => useCpuDebug(current, gameClient, true), {
      initialProps: { current: initial },
    })
    await act(async () => { await Promise.resolve() })
    expect(hook.result.current.type).toBe('ready')
    await act(async () => { await vi.advanceTimersByTimeAsync(100) })
    hook.rerender({ current: {
      ...initial, connection: 'reconnecting' as const, admissionPending: true,
    } })
    expect(hook.result.current.type).toBe('waiting')
    await act(async () => { await vi.advanceTimersByTimeAsync(100) })
    const resumed = { ...initial, admissionBaseline: { ...initial.admissionBaseline! } }
    hook.rerender({ current: resumed })
    expect(hook.result.current.type).toBe('waiting')
    expect(request).toHaveBeenCalledTimes(1)
    await act(async () => { await vi.advanceTimersByTimeAsync(299) })
    expect(request).toHaveBeenCalledTimes(1)
    await act(async () => { await vi.advanceTimersByTimeAsync(1) })
    expect(request).toHaveBeenCalledTimes(2)
    expect(starts).toEqual([0, 500])
    expect(hook.result.current.type).toBe('waiting')
    await act(async () => { resolveFresh(result(payload())); await fresh })
    expect(hook.result.current.type).toBe('ready')
  })

  it('retains the request-start gate across opt-in and leave-pending eligibility changes', async () => {
    vi.useFakeTimers()
    const starts: number[] = []
    const request = vi.fn().mockImplementation(() => {
      starts.push(performance.now())
      return Promise.resolve(result(payload()))
    })
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current, enabled }) => useCpuDebug(current, gameClient, enabled), {
      initialProps: { current: initial, enabled: true },
    })
    await act(async () => { await Promise.resolve(); await vi.advanceTimersByTimeAsync(100) })
    hook.rerender({ current: { ...initial, leavePending: true }, enabled: true })
    await act(async () => { await vi.advanceTimersByTimeAsync(50) })
    hook.rerender({ current: initial, enabled: false })
    await act(async () => { await vi.advanceTimersByTimeAsync(50) })
    hook.rerender({ current: initial, enabled: true })
    expect(request).toHaveBeenCalledTimes(1)
    await act(async () => { await vi.advanceTimersByTimeAsync(299) })
    expect(request).toHaveBeenCalledTimes(1)
    await act(async () => { await vi.advanceTimersByTimeAsync(1) })
    expect(starts).toEqual([0, 500])
  })

})

describe('CPU debug response acceptance', () => {
  it('drops an old-round response after the polling generation changes', async () => {
    vi.useFakeTimers()
    let resolveFirst!: (value: CpuDebugHttpResult) => void
    const first = new Promise<CpuDebugHttpResult>((done) => { resolveFirst = done })
    const request = vi.fn().mockReturnValueOnce(first).mockReturnValueOnce(new Promise(() => undefined))
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current }) => useCpuDebug(current, gameClient, true), {
      initialProps: { current: initial },
    })
    const nextSnapshot = snapshot(21, 2)
    nextSnapshot.room.configuration.controllers[1] = 'cpu'
    nextSnapshot.room.configuration.cpuDifficulty = 'standard'
    nextSnapshot.room.seats[1].controller = { type: 'cpu', difficulty: 'standard' }
    hook.rerender({ current: { ...initial, room: nextSnapshot.room, snapshot: nextSnapshot } })
    expect((request.mock.calls[0][0] as AbortSignal).aborted).toBe(true)
    await act(async () => { resolveFirst(result(payload())); await first })
    expect(hook.result.current.type).toBe('waiting')
    await act(async () => { await vi.advanceTimersByTimeAsync(500) })
    expect(request).toHaveBeenCalledTimes(2)
  })

  it('fails closed on malformed or out-of-order responses', async () => {
    vi.useFakeTimers()
    const malformed = { ...payload(), cpus: [{ seat: 'two', status: 'available', objective: 'secret' }] }
    const request = vi.fn()
      .mockResolvedValueOnce(result(payload(21)))
      .mockResolvedValueOnce(result(payload(20)))
      .mockResolvedValueOnce({
        ...result(payload()), response: new Response(JSON.stringify(malformed), { status: 200 }),
      })
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(() => useCpuDebug(initial, gameClient, true))
    await act(async () => { await Promise.resolve() })
    expect(hook.result.current.type).toBe('ready')
    await act(async () => { await vi.advanceTimersByTimeAsync(500) })
    expect(hook.result.current.type).toBe('error')
    await act(async () => { await vi.advanceTimersByTimeAsync(500) })
    expect(hook.result.current.type).toBe('error')
  })

  it('shows disabled after 404 and does not automatically retry', async () => {
    vi.useFakeTimers()
    const request = vi.fn().mockResolvedValue({
      ...result(payload()), response: new Response('{}', { status: 404 }),
    })
    const gameClient = client(request)
    const initial = state()
    const hook = renderHook(({ current }) => useCpuDebug(current, gameClient, true), {
      initialProps: { current: initial },
    })
    await act(async () => { await Promise.resolve() })
    expect(hook.result.current.type).toBe('disabled')
    const nextSnapshot = snapshot(21, 2)
    nextSnapshot.room.configuration.controllers[1] = 'cpu'
    nextSnapshot.room.configuration.cpuDifficulty = 'standard'
    nextSnapshot.room.seats[1].controller = { type: 'cpu', difficulty: 'standard' }
    hook.rerender({ current: { ...initial, room: nextSnapshot.room, snapshot: nextSnapshot } })
    await act(async () => { await vi.advanceTimersByTimeAsync(5_000) })
    expect(request).toHaveBeenCalledTimes(1)
    expect(hook.result.current.type).toBe('disabled')
  })
})
