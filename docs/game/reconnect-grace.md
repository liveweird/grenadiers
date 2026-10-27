# Pure reconnect-grace facts

`core::room::reconnect::ReconnectGrace` records two seats as Vacant, Connected,
Disconnected with an absolute expiry tick, or Expired. This component has no sockets,
credentials, serialization, or match-outcome policy. The active v13 owner integrates these facts
with private capabilities, lifecycle outcomes, and settled publication.

Occupy a vacant seat at the authoritative global tick. Its first `disconnect(seat, tick)`
sets a checked `tick + 600` deadline; repeated loss retains that deadline. `resume` succeeds
strictly before it. At the exact deadline it rejects without mutation; expired seats cannot
resume. `release` frees a reservation, including an already-expired seat.

Call `expire_due(tick)` on each accepted global boundary before admitting a resume. It
commits both due seats as one bounded batch and returns newly-expired flags plus resulting
states. The integrating lifecycle decides forfeiture or abandonment from those facts;
this API never awards a winner. Continue advancing deadlines through running and held rounds.

All successful operations, including duplicate loss and no-op expiry/release, advance a
monotonic room-lifetime high-water tick. Equal ticks support batched operations; older ticks
fail. State, clock, and seats remain unchanged after any error, including deadline overflow.
Callers must settle due authoritative boundaries before checking a resume; a failed operation
is not a substitute for advancing the accepted clock.

Tests cover the 599/600 boundary, duplicate/staggered loss, simultaneous expiry in both seat
orders, expired rejection, no-op time, invalid-state errors, stale operations, and atomic
overflow. Run `cargo test -p grenadiers-core room::reconnect` or `scripts/gates.sh server`.
