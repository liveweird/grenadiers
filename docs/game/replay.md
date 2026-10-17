# Private deterministic replay evidence

The test-only `server::game::rooms::replay` module replays semantic operations through the
real MatchRuntime, InputMailbox, generated-world profile, and entropy seam. It does not run
a second simulator. Live capture is a separate increment; no endpoint or persistent journal
is available in this release.

A versioned header identifies format 1, M5 lifecycle semantics, classic-v2 physics,
generator-v1, initial occupied seats, and initial absolute tick zero. Operations record the
authoritative tick and increasing order within that tick: occupation, accepted readiness/
movement/bomb/rematch, immediate input fencing, tick advancement, committed departure batches,
and interruption. Actual-start markers contain the real epoch, assignment, and private
GenerationRecord. Outcomes are recomputed, never injected.

The runner rejects unsupported profiles, time/order errors, invalid bomb action bounds,
missing/extra seeds, mismatched assignments, missing start markers, and truncated journals.
Every actual round consumes its own recorded seed; canceled countdowns consume none.
Movement refreshes and immediate input clearing are significant: recording only a later
lifecycle departure cannot reproduce a vulnerable running character or queued bomb.

The journal preallocates 60,000 records, measured at 64 bytes each on the verification target
(3,840,000 bytes of record capacity, below the 4 MiB journal budget). Header/container overhead
fits the remaining space. Once full, it records `Truncated { last_complete_index }` and stops
growing; the runner refuses to certify a truncated replay as complete. Unbounded draws cannot
have guaranteed full retention in a finite buffer. Replay working state is separate from this
journal storage budget.

Records contain no session/resume credentials, raw frames, socket IDs, or public room codes.
Generation records retain private seeds; their Debug output is redacted. Journal Debug shows
only header, record count, and status. Private test checkpoints compare complete State,
MatchLifecycle/series, input mailbox contents, pending StepReports, and revision.

Run `cargo test -p grenadiers-server game::rooms::replay`, then `scripts/gates.sh server`
and `scripts/gates.sh coverage`. Fixtures exercise generated draws, three real wins, rematch,
cancellation, interruption, TTL refresh, and input-fence ordering, plus malformed journals.
