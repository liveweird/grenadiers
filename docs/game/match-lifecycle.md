# Pure match lifecycle

`core::room::MatchLifecycle` composes occupancy, readiness, the absolute clock, command epochs,
and `MatchSeries`. The production v6 match owner and runtime use this lifecycle. The archived
`RoomLifecycle` remains covered for the historical single-round model. This API owns no
physics or I/O.

Occupy both seats and call `set_ready`. Two ready players start an initial 180-tick countdown.
Changing readiness cancels it, clears both votes, and advances the command epoch once without
consuming an actual round. `advance_idle_tick()` returns `RoundStarted(assignment)` exactly at
the deadline; the owner constructs fresh physics at that tick before its first update.

During Running, commit each accepted update with `complete_running_tick(result)`. A nonfinal
result enters a 180-tick hold, then automatically advances the epoch and starts the next
180-tick countdown. Its start advances actual round assignment. Physics supplies round timeout;
the lifecycle does not fabricate results. Reaching the configured win target enters MatchResult
immediately; the default target is three.

Pass departures from one authoritative tick together to `depart_batch`, after that tick's
simulation/scoring. Explicit departures before the first start cancel admission/countdown;
afterward they forfeit or abandon, while transport loss interrupts. A finalized match remains
final. `interrupt` handles the four nonparticipant error/shutdown reasons; explicit departures
must use the batch API. Prematch interruption has no invented match outcome.

The active reconnect path uses `depart_with_grace` with the complete two-seat boundary batch.
Before the first actual start, every departure releases its seat immediately. After start,
transport loss reserves the participant for 600 authoritative ticks and duplicate loss keeps
the original deadline. `resume_connection` succeeds strictly before that seat's deadline.
`expire_reconnect_grace` records due reservations as expired and applies the match decision
table without inventing a round result: a connected opponent wins a single expiry, two expired
participants abandon, and an expired participant waits while the opponent remains eligible but
disconnected. Explicit departure requires a connected seat; an eligible disconnected opponent
still wins that forfeit. The historical `depart_batch` behavior remains covered by profile-1 replay.

Result holds last at least 180 ticks and remain in Result while either reservation is not
connected. Inter-round loss cancels the countdown once, advances the command epoch without
consuming an actual round, and preserves the completed round and original hold deadline. After
both participants return, the next idle tick starts a fresh full countdown. Running callers
commit physics and scoring first, then pass the whole departure batch and settle expiry. Idle
callers pass loss before the idle transition and settle expiry after it. This ordering lets a
winning round precede expiry and prevents a partial loss batch from fabricating a forfeit.

Global time continues while awaiting rematch. After `rematch_at`, occupied players may set or
withdraw rematch votes. Departures invalidate their votes; vacant opponents cannot complete
consensus. Both eligible votes advance the command epoch once and return occupied, unready
seats to Waiting with cleared series facts. The owner separately resets worlds, inputs, and
lobby deadlines while retaining socket sequences and binding identities.

Grace reservations and their monotonic time high-water survive a rematch reset. Only connected
participants may ready or vote for rematch. A terminal reservation created before finality may
resume until its original deadline, while a new terminal transport loss releases immediately
and never changes the finalized outcome.

All fallible transitions clone only bounded state and commit after validation, so errors leave
clock, seats, series, epoch, and phase unchanged. Run
`cargo test -p grenadiers-core room::match_lifecycle --locked` for exact boundaries, complete
score/draw progression, departure precedence, rematch consent, and overflow regressions.
