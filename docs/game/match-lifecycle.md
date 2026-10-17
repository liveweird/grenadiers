# Pure match lifecycle

`core::room::MatchLifecycle` composes occupancy, readiness, the absolute clock, command epochs,
and `MatchSeries`. The production v3 match owner and runtime use this lifecycle. The archived
`RoomLifecycle` remains covered for the historical single-round model. This API owns no
physics or I/O.

Occupy both seats and call `set_ready`. Two ready players start an initial 180-tick countdown.
Changing readiness cancels it, clears both votes, and advances the command epoch once without
consuming an actual round. `advance_idle_tick()` returns `RoundStarted(assignment)` exactly at
the deadline; the owner constructs fresh physics at that tick before its first update.

During Running, commit each accepted update with `complete_running_tick(result)`. A nonfinal
result enters a 180-tick hold, then automatically advances the epoch and starts the next
180-tick countdown. Its start advances actual round assignment. Physics supplies round timeout;
the lifecycle does not fabricate results. The third win enters MatchResult immediately.

Pass departures from one authoritative tick together to `depart_batch`, after that tick's
simulation/scoring. Explicit departures before the first start cancel admission/countdown;
afterward they forfeit or abandon, while transport loss interrupts. A finalized match remains
final. `interrupt` handles the four nonparticipant error/shutdown reasons; explicit departures
must use the batch API. Prematch interruption has no invented match outcome.

Global time continues while awaiting rematch. After `rematch_at`, occupied players may set or
withdraw rematch votes. Departures invalidate their votes; vacant opponents cannot complete
consensus. Both eligible votes advance the command epoch once and return occupied, unready
seats to Waiting with cleared series facts. The owner separately resets worlds, inputs, and
lobby deadlines while retaining socket sequences and binding identities.

All fallible transitions clone only bounded state and commit after validation, so errors leave
clock, seats, series, epoch, and phase unchanged. Run
`cargo test -p grenadiers-core room::match_lifecycle --locked` for exact boundaries, complete
score/draw progression, departure precedence, rematch consent, and overflow regressions.
