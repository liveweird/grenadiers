# Bounded match owner

Status: active in the existing registry and v4 endpoint. Authentication, sequence ownership,
and connection generations remain in their original transport boundaries.

One task owns the real MatchRuntime, a 32-command queue, two generation-bound capability slots,
and at most two pending departures/leave replies. Submissions never await inside the owner. A leave
is validated and its seat's input fenced immediately; occupancy and acknowledgment change
only when the scheduled batch commits. Pending seats cannot issue further commands, and
pending departures block admission/rematch. The first accepted reason per seat is retained.

Join and resume use synchronous preparation, a fallible reply send, then an infallible commit
without an intervening await. Canceled replies mutate nothing. Restore keeps a stable credential;
connected takeover fences accepted input, preserves bomb high-water, installs a fresh exact
connection/generation binding, then signals the old socket. Old commands and delayed drops cannot
affect the replacement. Unavailable resume attempts uniformly return `resumeRejected`.

Running boundaries commit physics and scoring before departures; all idle phases apply
departures before automatic transitions, stamped at the last completed tick. Countdown-boundary
leaves therefore consume no new actual round or entropy. Final match outcomes remain final.

Scheduling retains absolute 60 Hz deadlines, 20 Hz publication, four-update catch-up tranches,
and a 500 ms overload limit. If catch-up needs another tranche, the owner retains the same bounded
batch and admits no later command into overdue ticks. The 300-second lobby deadline applies before the first actual
start and restarts after rematch; long matches survive their creation deadline. Only final
matches/prematch interruptions begin fixed 30-second retirement; nonfinal results do not.

Sticky shutdown bypasses full queues. Shutdown/overload atomically interrupt, apply pending
departures, and publish once, preserving reports, vacancy, and prior match finality. Leave
replies complete only after that checked publication. The watch retains `Closing {last, reason}`
for slow readers. Cleanup is guarded before spawning, including abort-before-first-poll.

Sockets deliver the retained closing publication before close, best effort within five seconds.
The process explicitly drains upgraded game/diagnostic sockets and room cleanup before exit. The adapter
never duplicates authentication or introduces a selectable protocol/production testing flag.
