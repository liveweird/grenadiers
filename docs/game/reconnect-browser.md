# Reconnect browser

Status: active. The application uses the live gameplay client ([v11](protocol-v11.md)) against
the existing gameplay endpoint. The client retains the reconnect lifecycle introduced in v4 and [shared settings](settings.md).

## Private admission state

The client first obtains a normal single-use HTTP session and authenticates a WebSocket with
the live protocol version. A personal `admitted` frame establishes the seat, stable resume
capability, authoritative room view, nullable full physical state, and accepted bomb-action high-water. The
client turns an admitted physical body into a snapshot with an empty event list; missed events are
not replayed. Shared `roomState` frames cannot establish identity.

The resume capability exists only in the `GameClient` instance. It is absent from public React
state, URLs, logs, local storage and session storage. Explicit leave acknowledgement, terminal
recovery failure, and disposal clear it. A normal epoch change resets bomb action IDs; a resumed baseline instead
continues above `lastActionId`. A high-water of `2147483647` leaves bomb actions exhausted for
that epoch. Commands sent before a disconnect are never buffered or resent.

## Recovery and fencing

An unexpected close with a capability starts one recovery episode with a ten-second monotonic
deadline and at most three reconnect attempts. Every transport attempt obtains a fresh HTTP
session, authenticates, and only then sends `resumeRoom`. Fetch, socket and
message callbacks carry a connection generation so callbacks from replaced attempts cannot change
state. Shared room and snapshot frames received while the personal baseline is pending are ignored.
The resumed admission must keep the same room, seat and capability and cannot move revision or
command epoch backward.

A correlated `rateLimited` response to the pending resume keeps authentication, capability and
the last baseline. The client retries on that socket after 250, 500 and 750 milliseconds, using
a fresh increasing sequence each time. Those three additional retries are shared across the
episode; reconnecting or authenticating does not reset the count or deadline. Initial create/join
throttles settle for manual retry on the authenticated socket. Rejected gameplay actions are
never automatically replayed.

At the deadline, pending fetches and timers are canceled, the socket closes, and recovery ends
with `connectionFailed`. A late admission or throttle cannot bypass that deadline. Exhausting
same-socket retries ends with `rateLimited`; other correlated resume errors terminate recovery.
This local time budget does not guarantee how much authoritative seat reservation remains.
Timer-registration tokens and connection generations fence callbacks already queued at cancellation.

The last authoritative room and world remain visible during recovery, including an authenticated
socket still awaiting personal admission. Readiness, rematch, settings, Leave, movement and bomb
controls stay disabled. The shared arena controller already releases held input
when disabled, blurred, hidden, or replaced by a new room/epoch identity. Controls return only after
the admitted full baseline arrives, so a hidden tab does not replay held movement on return.

`resumeRejected`, `seatReplaced`, WebSocket close code 4001, and exhausted retries stop automatic
recovery, discard personal room identity, and return the panel to a usable lobby with the failure
still visible. The client does not compete with the new seat owner. A rejected leave retains its
capability and authority; an acknowledged leave clears both. Transport loss while leave is pending
honors the explicit leave locally instead of blindly resuming a possibly released seat. Explicit
disposal also cancels session work and retry timers. Ordinary stale revision, command-epoch and
obsolete movement-error protections remain.

## Presentation

The active panel reuses the existing Pixi courtyard and match layout. It renders disconnected and
expired seat facts, an accessible grace timer, a reconnect status, and a completed result that is
held beyond its normal deadline while a reserved player is away. Scores and the last world remain
authoritative; connected-only controls stay disabled during recovery. English and Polish copy use
the same keys and describe the action available to the player.

The server parser and application client use the live contract together. Earlier versions remain archived and are
not accepted by the active authenticated gameplay socket.
