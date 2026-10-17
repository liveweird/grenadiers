# ADR 0010: Authoritative room scheduling and publication

Status: implemented for the fixed-map two-player preview.

## Decision

One Tokio task owns each room's lifecycle, simulation, membership, input mailboxes, and
publication revision. Use absolute rational 60 Hz deadlines from room creation rather than
adding truncated intervals or advancing from wall-clock deltas. Complete at most four overdue
updates before yielding; skip none. More than 500 ms behind the next deadline interrupts a
nonterminal round. An existing result remains final during overload.

Process a bounded command batch before due work. Each seat independently admits at most
eight movement/bomb commands per update; queue and socket limits bound the remaining work.
A new direction presses before releasing the old one. An unchanged refresh preserves the
simulation's fractional movement. Expire held movement after 15 updates without refresh.
Coalesce accepted fresh bomb actions to one placement per seat/update, appended after all
movement/clear inputs. Consume the action ID even if capacity or placement later prevents it.

At the countdown boundary construct the simulation at the lifecycle's absolute tick with
zero running updates. Accept the first physics update on the next tick. Once an update begins,
complete the simulation step and commit its lifecycle result synchronously before processing
departures received during it. A departure already observed before accepting that update may
interrupt at the previous completed tick. Canceled countdowns fence delayed input by attempt.

## Publication and lifetime

Publish membership/readiness and phase transitions immediately. Publish Countdown room views
and Running/Result snapshots every three ticks. Check the prospective revision, matching
lifecycle/state clocks and result, up to three consecutive reports/64 events, and the 64 KiB
encoded frame limit before replacing the watch value. Both seats share identical encoded
snapshot bytes; only admission views carry personalized seat identity. Slow sockets consume
the latest publication without blocking simulation. Never truncate events to fit a frame.

Waiting/Countdown expires 300 seconds after room creation; Running removes that deadline.
A terminal room retires 30 seconds after its first terminal transition, without activity
extending retention. Empty rooms release immediately. Keep registration cleanup in a guard
constructed before task spawning so cancellation before the first poll also releases discovery.
Process restart loses active rooms; reconnect, recovery, and multi-replica routing remain future
milestones. See [room behavior](../game/rooms.md) and [protocol](../game/protocol.md).
