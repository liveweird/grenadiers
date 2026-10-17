# Room lifecycle

Status: public `core::room` lifecycle plus server-owned networked Waiting rooms. Anonymous
HTTP credentials are separate from occupancy; a successful create/join binds a seat.

## Admission and readiness

A new `RoomLifecycle` starts at tick zero, round attempt one, with two vacant/unready seats
in Waiting. `occupy(seat)` admits only a vacant Waiting seat. `set_ready(seat, ready)` accepts
only occupied Waiting/Countdown seats; repeating the current value changes nothing.
Both occupied and ready starts Countdown, ending at the current tick plus 180.

`advance_idle_tick()` advances Waiting, Countdown, and Result. Countdown enters Running
exactly at its deadline and clears readiness. This transition performs zero physics updates;
the first `complete_running_tick(result)` belongs to the following tick. That method accepts
only Running and records a supplied completed simulation outcome at the new tick.

## Cancellation and finality

Unready or departure during Countdown returns to Waiting, clears both ready flags, and
increments the round attempt once. A Waiting departure clears readiness without advancing
the attempt. Repeated departure of a vacant seat changes nothing.

During Running, explicit departure interrupts with `participantLeft`; transport loss uses
`connectionLost`, both at the last completed tick. A completed result committed first remains
final after either departure reason. Server overload/stop/internal-error interruption is also
explicit. Result/Interrupted reject new occupants and readiness; terminal departures can
remove occupancy without changing the outcome or end tick. Interrupted clocks stay frozen.

All fallible clock/deadline/attempt additions are checked before mutation. Errors preserve the
entire lifecycle. The server will additionally enforce wire-safe counters/ticks and bind each
seat to authenticated connection ownership; domain seat arguments are not client authority.

## Integration boundary

The future room owner must complete and record an accepted simulation update before handling
pending Running departures. Process admitted cancellation before the countdown boundary.
Keep discovery, credentials, membership generations, input deduplication, bounded channels,
60 Hz scheduling, and 20 Hz publication outside this pure state machine. See
[ADR 0009](../decisions/0009-room-lifecycle.md) and the [protocol](protocol.md).

## Network admission

After gameplay authentication, `createRoom` claims seat one and returns a personalized
`roomState`. `joinRoom` uses the public eight-character code and claims the vacant seat.
Both sockets observe the same room view; only `youSeat` differs. Admission arrives before
subsequent membership updates. Codes are public discovery identifiers, not credentials.

The process holds at most 32 rooms. Create/join share a burst of 32 operations refilling at
eight per second. Each owner has an eight-command mailbox; slow sockets consume only the
latest watch update. OS randomness supplies codes and a process ID prefix, with bounded
collision retries and checked serials. No room state is persisted or shared across replicas.

Connection ID plus admission generation fences membership. Explicit `leaveRoom` validates
the room/attempt, removes occupancy, then sends the correlated `leftRoom`; the authenticated
socket can create/join again. Transport loss has an independent departure channel, so a full
command queue cannot strand a seat. Empty rooms release discovery immediately. Registry
cleanup also runs when an owner task is canceled or panics.

Waiting rooms expire 300 seconds after creation, independent of activity. Expiry sends an
uncorrelated `roomNotFound` error and closes the socket; it never fabricates `leftRoom`.
Bound readiness/movement/bomb commands currently return `invalidPhase`; authoritative
countdown and simulation integration follow separately.
