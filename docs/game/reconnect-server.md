# Reconnect server behavior

The gameplay endpoint (live contract [v11](protocol-v11.md)) keeps one private resume capability
for each occupied seat. A transport loss changes that seat to `disconnected` and reserves it through the published
`expiresAt` tick. The capability stays stable across restore, connected takeover, and rematch. An
explicit leave releases the seat immediately. Capabilities are independently generated, redacted
from logs and debug output, and never appear in shared room publications or replay records.

Create, join, and resume admissions carry the client's transport sequence. The owner prepares the
full personal `admitted` response without changing the room, sends it through the admission
oneshot, and commits only after that send succeeds. A canceled admission therefore cannot occupy a
seat, replace a connection, consume input, or rotate a credential. Failure while writing an already
committed admission is treated as a transport loss and starts the normal reservation window.

A restore changes a disconnected reservation back to connected. A takeover of an already
connected seat first fences queued movement and bomb intent, then installs a fresh binding. The old
socket receives `seatReplaced` and closes with code 4001. Commands and delayed drops include the
exact connection ID and generation; an old binding cannot act on or disconnect its replacement.
The personal admission includes the authoritative room, optional current physics state, and bomb
action high-water for the current physics epoch. Reading it does not consume pending reports or
change shared watch history.

The room owner settles each simulation boundary before processing deferred admissions. Running
boundaries apply physics and scoring before the complete loss batch and reservation expiry; idle
boundaries apply losses before advancing the lifecycle clock. Expiry occurs at the exact published
deadline, so a restore is accepted on the preceding tick and rejected once that deadline is
settled. Catch-up remains bounded to four ticks before yielding, and all command queues retain their
existing bounds.

Completed physics remains private during an inter-round countdown and can be restored if that
countdown is canceled. Shared countdown publications contain no stale physics body. Terminal rooms
keep only eligible reservations and their original deadlines; they do not extend grace on another
loss. Shutdown interrupts and settles accepted departures before the final closing publication,
within the process-wide five-second drain budget.

Private semantic replay profile 2 records grace departures, restore or replacement, expiry, and
report flushes in lifecycle order. It records no capability, binding, socket frame, or session
identity. The journal remains bounded to 60,000 records and less than 4 MiB; truncation or an
unreproducible round-construction failure makes it explicitly incomplete and prevents a certified
complete replay.
