# ADR 0009: Pure room lifecycle before transport ownership

Status: accepted; the headless lifecycle is implemented. Server room ownership follows.

## Decision

Keep two-seat occupancy, readiness, countdown, and terminal phase transitions in public
`core::room::RoomLifecycle`, independent of Serde, Tokio, network credentials, discovery,
and publication. Reuse domain seat/tick/result types. The server remains the sole authority
that will own this lifecycle alongside the simulation; the browser does not advance it.

Only Waiting admits occupants. Both ready starts exactly 180 idle ticks of Countdown;
entering Running performs no physics update. The first completed simulation update belongs
to the following tick. Canceling Countdown clears both readiness flags and advances the
attempt once, allowing the future transport to reject delayed commands from old attempts.

Record a completed running result before processing pending departures. A Running departure
interrupts the preview without inventing a winner. Result and Interrupted are immutable;
terminal departures only remove occupancy. Result clocks may advance while their end tick
stays fixed; Interrupted freezes. Checked additions occur before mutation.

## Consequences

The documented headless API and deterministic tests make lifecycle transitions reviewable
before asynchronous ownership is introduced. Connection IDs, binding generations, room IDs,
join codes, revisions, bounded channels, random generation, and scheduler policy remain server
responsibilities. Add the absolute-start simulation constructor only with its real room-loop
consumer. Publication must enforce wire counter/tick limits in addition to domain arithmetic.

[Room guidance](../game/rooms.md) describes the current API and temporary preview semantics.
