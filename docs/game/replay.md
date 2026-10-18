# Private deterministic replay evidence

The production MatchRuntime records accepted semantic operations in a private in-memory
journal. The test-only replay runner drives the real MatchRuntime, InputMailbox, generated-world
profile, and entropy seam; it does not run a second simulator. Replay inspection remains
test-only. No endpoint, public export, or persistent journal is available.

A versioned header identifies format 2, lifecycle profile 2, Expanded-v5 physics and generator-v2.
Archived format-1 journals retain configurable-v4/classic-v2/v3 physics and generator-v1.
Both formats retain
initial occupied seats, and initial absolute tick zero. Operations record the
authoritative tick and increasing order within that tick: occupation, accepted readiness/
movement/bomb/rematch, immediate input fencing, tick advancement, committed departure batches,
grace departure, restore/replace, expiry, explicit report flush, and interruption, including one
atomic interrupt-plus-depart operation. Actual-start markers contain the real epoch, assignment,
and private GenerationRecord. Outcomes are recomputed, never injected. A test-only profile-1
adapter replays archived journals through the same runtime and physics helpers.

The runner rejects unsupported profiles, time/order errors, invalid bomb action bounds,
missing/extra seeds, mismatched assignments, missing start markers, and truncated/incomplete journals.
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
MatchLifecycle/series, input mailbox contents, and pending StepReports. Checkpoints retain
complete ordered pending manual queues, including repeated requests; the test-only inspection
representation is not a serialized journal contract. Transport publication
revision is excluded: initial/final notices may advance it without changing gameplay. Profile 2
records every report-clearing publication as `ReportFlush`; personal admission does not consume
reports.

Run `cargo test -p grenadiers-server game::rooms::replay`, then `scripts/gates.sh server`
and `scripts/gates.sh coverage`. Fixtures exercise generated draws, three real wins, rematch,
cancellation, interruption, grace restore/expiry, TTL refresh, input-fence ordering, and an
archived profile-1 gameplay result, plus malformed journals.

Capture happens only after successful semantic mutation, including unchanged readiness and
same-direction TTL refreshes. Wrong epoch/phase, rate-limited, duplicate, and out-of-wire-range
bomb commands create no records. Input fencing remains separate from later boundary departure.
Replay disables its own capture privately to avoid recursive journals.

The profile construction API reports unavailable entropy or failed construction as the same
absence. Capture therefore marks `Incomplete { reason: RoundConstruction }` after the truthful
interrupted tick; replay refuses completeness instead of inventing a seed or world. Fixed-v1
unit fixtures are explicitly unsupported. Journal overhead plus actual allocated record
capacity measures 3,840,056 bytes on the verification target. Owner destruction frees it;
32 bounded rooms retain below 128 MiB of journal storage. No restart recovery is promised.

Configurable journals also retain initial settings, settings revision and editor identity. Accepted
revision-fenced readiness/settings actions and permanent authority release replay deterministically.
Legacy journals reject v6-only actions; configurable journals reject legacy unfenced readiness.
Snapshot publication and personal admission bind the complete immutable settings identity.

Expanded journals retain the enabled-item mask inside complete settings. They do not add a
duplicate mask to round-start generation records. Format 2 accepts only the supported rules-5,
generator-2 and M6 lifecycle tuple; older profiles require format 1 and empty item masks.
Replay regenerates from the recorded seed and accepted configuration and compares the actual
generation record. Heart enable/disable, reveal, collection, consumption and later elimination
must reproduce the complete state, grace and event reports.
