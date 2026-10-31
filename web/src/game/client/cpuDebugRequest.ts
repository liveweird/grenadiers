import type { GameClientState } from './clientTypes'
import type { Capability, Dependencies } from './clientProtocol'
import type { CpuDebugHttpResult, CpuDebugRequest } from './cpuDebugTypes'

export async function requestCpuDebug(
  fetcher: Dependencies['fetch'],
  state: GameClientState,
  capability: Capability | null,
  isCurrent: () => boolean,
  signal: AbortSignal,
): Promise<CpuDebugHttpResult> {
  const room = state.room
  const own = room?.seats.find(({ seat }) => seat === state.youSeat)
  if (!room || !capability || state.youSeat !== capability.seat
    || room.roomId !== capability.roomId || state.connection !== 'authenticated'
    || state.admissionPending || state.leavePending || own?.controller.type !== 'human'
    || own.connection.type !== 'connected') {
    return { type: 'unavailable' }
  }
  const baseline = {
    roomId: room.roomId, roundId: room.roundId, tick: room.tick,
    revision: room.revision, youSeat: capability.seat,
  }
  const body: CpuDebugRequest = { roomId: room.roomId }
  const response = await fetcher('/api/v1/game/cpu-debug', {
    method: 'POST',
    headers: {
      Accept: 'application/json',
      Authorization: `Bearer ${capability.token}`,
      'Content-Type': 'application/json',
    },
    body: JSON.stringify(body),
    cache: 'no-store',
    credentials: 'omit',
    signal,
  })
  return isCurrent() ? { type: 'response', response, baseline } : { type: 'stale' }
}
