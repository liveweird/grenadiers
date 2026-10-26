# Private deterministic replay evidence

Production `MatchRuntime::new` allocates no journal and skips replay-only capture work.
Explicit `new_recording` test fixtures retain real bounded journals; the test-only runner drives
the same runtime, mailbox, generated-world profile and entropy seam. Replay inspection is test-only.
There is no operator toggle, endpoint, public export or persistent journal.
[ADR 0025](../decisions/0025-production-replay-recording.md) defines this allocation policy.

A versioned header identifies format 2, lifecycle profile 2, Expanded-v5 physics and generator-v2.
Archived format-1 journals retain configurable-v4/classic-v2/v3 physics and generator-v1.
Solo format 3 adds mode, CPU seat/profile/policy version and private CPU seed provenance to the
header, retaining lifecycle profile 2, Expanded-v5 physics, generator-v2 and the same record
capacity. Formats 1/2 remain human-duel journals. The runner constructs no CPU worker and never
reruns a policy: controller readiness, movement, manual input, scheduled Stop and rematch are
ordinary recorded operations. CPU seed Debug output is redacted.
All formats retain
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

An explicitly recorded test runtime preallocates 60,000 records, with a 64-byte record on the
verified target (3,840,000 bytes of capacity, below the 4 MiB journal budget). Production rooms
reserve no journal vector capacity. These are record-storage bounds, not whole-room memory or RSS. Header/container overhead
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

When explicitly enabled in tests, capture happens only after successful semantic mutation, including unchanged readiness and
same-direction TTL refreshes. Wrong epoch/phase, rate-limited, duplicate, and out-of-wire-range
bomb commands create no records. Input fencing remains separate from later boundary departure.
Replay disables its own capture privately to avoid recursive journals.

The profile construction API reports unavailable entropy or failed construction as the same
absence. Capture therefore marks `Incomplete { reason: RoundConstruction }` after the truthful
interrupted tick; replay refuses completeness instead of inventing a seed or world. Fixed-v1
unit fixtures remain explicitly unsupported for recording. Disabled recording is absence of
evidence, not an empty complete replay. Recorded-fixture destruction frees its journal. Live
publication, private generation records, recipient filtering and reconnect state remain intact;
none depends on keeping a semantic journal. No restart recovery is promised.
Measured allocation/timing evidence is preserved in the
[allocation history](../history/replay-allocation.md).

Configurable journals also retain initial settings, settings revision and editor identity. Accepted
revision-fenced readiness/settings actions and permanent authority release replay deterministically.
Legacy journals reject v6-only actions; configurable journals reject legacy unfenced readiness.
Snapshot publication and personal admission bind the complete immutable settings identity.

Oil Barrel keeps journal format 2 unchanged. Effective settings and the private generation record
reconstruct every concealed, pending, unlit and burning site; complete state equality also includes
typed Oil exit grace and persistent fire. Replay neither adds an Oil operation nor serializes a
public pickup/event surrogate. Reconnect, pause, terminal freeze and new-round reset therefore
follow the same accepted inputs, ticks and generation record as live execution.

Expanded journals retain the enabled-item mask inside complete settings. They do not add a
duplicate mask to round-start generation records. Format 2 accepts only the supported rules-5,
generator-2 and M6 lifecycle tuple; older profiles require format 1 and empty item masks.
Replay regenerates from the recorded seed and accepted configuration and compares the actual
generation record. Heart/Shield enablement, reveal, collection, immunity, consumption, expiry and later elimination
must reproduce the complete state, grace and event reports.
