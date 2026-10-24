# Gameplay v4 resumable match contract

Status: archived; the live contract is [v11](protocol-v11.md). V4 defined the checked contract for
reconnect grace and stable seat capabilities; it added no endpoint or feature flag.

V4 retains the v3 physical `GameState` and event definitions. Authentication success is fixed
to protocol version 4, while authentication input remains bounded to 0..65,535 for negotiation.
The client adds `resumeRoom {sequence, roomId, resumeToken}`. A resume token is a distinct
64-character lowercase hexadecimal capability and must not be placed in shared room notices,
URLs, logs, replay journals, seeds, or ordinary session-token storage.

## Personal and shared messages

`admitted` is the only room baseline that carries personal identity and capability data. It
contains the acknowledged sequence, `youSeat`, a resume token, nullable bomb-action high-water,
and a nullable physical state. It never contains events. `roomState` contains only the shared
room view. `seatReplaced {roomId}` tells an older owner binding to stop reconnecting.

`RoomView.seats[*].connection` is one of vacant, connected, disconnected with an exclusive
`expiresAt`, or expired. `occupied` is true exactly when the connection is not vacant. Ready
and rematch votes require a connected seat. Disconnect deadlines are at most 600 ticks ahead;
terminal matches cannot create a new grace period after their outcome time.

| Phase | Physical baseline | Relevant timing |
| --- | --- | --- |
| waiting | none | fresh match facts |
| initial countdown | none | connected ready seats; 1..180 ticks ahead |
| running | required | state epoch and tick equal the current room epoch and tick |
| result | required completed body | `holdUntil = lastRound.endedAt + 180` |
| inter-round countdown | none | completed hold elapsed; new command epoch |
| match result | normally required | `rematchAt = endedAt + 180` |
| prematch interrupted | none | interruption tick equals room tick |

A result may remain published after `holdUntil` while a participant is disconnected. If an
inter-round countdown is canceled, the room returns to result with a newer command epoch while
the retained physical body remains bound to `lastRound.roundId`. This separation prevents stale
commands from being accepted merely because an old completed world is still visible.

## Checked projection

`project_room_view(RoomMetadata, &MatchLifecycle)` derives room facts directly from the lifecycle
and settled reservation state. `validate_room_view(&RoomView)` independently checks canonical
seat order, occupancy/connection agreement, readiness, grace bounds, score and completed-round
history, phase timing, and terminal results. Running and result phases keep both reservations;
an expired seat may wait only opposite a still-disconnected seat. A forfeit loser must be vacant
or expired, abandonment requires both seats vacant or expired, and later winner departure does
not erase the historical outcome.

`validate_state_binding(room, state_epoch, projected_round)` checks the output of the existing
physical `project_state` function. Running bodies bind to the current epoch and tick. Completed
result bodies bind to the latest completed round, including canceled countdowns. An unfinished
terminal body binds to the current epoch and outcome tick. Waiting, countdown, and prematch
interrupted phases reject physical bodies.

`validate_admission` additionally requires the admitted seat to be connected. It checks explicit
private state-epoch provenance and nullable-body rules. A null match-result body is accepted only
for an internal-error construction failure. A bomb-action high-water may be present only when the
body belongs to the current command epoch. Its numeric value comes from the owner's input mailbox;
the validator deliberately does not infer it from public bombs.

The schema enforces closed wire structure and numeric bounds. Semantic projection and validation
enforce lifecycle meaning. The server owner and browser client use these contracts together.
