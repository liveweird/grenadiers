# Room lifecycle

This document retains the original single-round domain model. Production now composes the
[match lifecycle](match-lifecycle.md) and [bounded match owner](match-owner.md) using
[gameplay v3](protocol.md); their match-result, rematch, and retention rules take precedence.


Status: public `core::room` lifecycle plus server-owned networked two-player rounds. Anonymous
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

The room owner completes and records an accepted simulation update before handling
pending Running departures. Process admitted cancellation before the countdown boundary.
Discovery, credentials, membership generations, input deduplication, bounded channels,
60 Hz scheduling, and 20 Hz publication remain outside this pure state machine. See
[ADR 0009](../decisions/0009-room-lifecycle.md) and the [protocol](protocol.md).

## Network admission

After gameplay authentication, `createRoom` claims seat one and returns a personalized
`roomState`. `joinRoom` uses the public eight-character code and claims the vacant seat.
Both sockets observe the same room view; only `youSeat` differs. Admission arrives before
subsequent membership updates. Codes are public discovery identifiers, not credentials.

The process holds at most 32 rooms. Create/join share a burst of 32 operations refilling at
eight per second. Each owner has a 32-command mailbox; slow sockets consume only the
latest watch update. OS randomness supplies codes and a process ID prefix, with bounded
collision retries and checked serials. No room state is persisted or shared across replicas.

Connection ID plus admission generation fences membership. Explicit `leaveRoom` validates
the room/attempt, removes occupancy, then sends the correlated `leftRoom`; the authenticated
socket can create/join again. Transport loss has an independent departure channel, so a full
command queue cannot strand a seat. Empty rooms release discovery immediately. Registry
cleanup also runs when an owner task is canceled or panics.

Waiting/Countdown rooms expire 300 seconds after creation, independent of activity; entering
Running removes that creation deadline. Expiry sends an
uncorrelated `roomNotFound` error and closes the socket; it never fabricates `leftRoom`.
Bound readiness controls Waiting/Countdown. Movement and bomb intents require Running;
invalid phases return `invalidPhase`. Join only admits Waiting rooms.

## Authoritative scheduling and input

One owner task uses absolute rational 60 Hz deadlines. It drains bounded admitted work,
completes at most four overdue updates before yielding, and never skips a simulation tick.
More than 500 ms behind the next deadline interrupts a nonterminal round with
`serverOverloaded`. Result finality survives lag. Waiting membership/readiness changes publish
immediately; Countdown views and Running/Result snapshots publish every three ticks, with
immediate phase transitions. A single prechecked encoded snapshot is shared by both seats.

Both Ready starts 180 countdown ticks. At the exact boundary the owner constructs the fixed
arena at the room's absolute tick with zero running updates; physics starts on the next tick.
Canceling clears both flags and advances the attempt. Commands, including Leave, validate
the current attempt rather than an admission-time copy. No input carries into another attempt.

Each seat can admit eight movement/bomb commands per update. A ninth rate-limits the offender;
the other seat keeps its own budget. Unchanged direction refreshes preserve fractional movement.
Fresh directions press before releasing the previous direction. Held input lasts 15 updates
without refresh; null clears it. Fresh monotonic bomb action IDs are consumed on admission,
including coalesced or unsuccessful placements. At most one placement intent per seat/update
runs after all movement/clear inputs, so a same-tick stop cannot cancel an accepted bomb.

The owner commits a completed simulation result before handling departures that arrived during
that accepted update. An already-observed departure may interrupt before accepting the next
update. Result worlds freeze while their room clock advances; Interrupted freezes entirely.
Terminal rooms retire after 30 seconds without activity extending retention. Empty rooms
release immediately. Terminal retirement closes normally without inventing an expiry error.

Publication validates the next revision, matching lifecycle/state tick and result, at most
three consecutive reports/64 events, and the 64 KiB encoder limit before replacing the watch
value. Failures preserve the last valid publication and retire the unavailable owner; events
are never truncated to make a frame fit.
