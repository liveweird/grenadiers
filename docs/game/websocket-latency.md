# Ordered WebSocket delay measurement

The M7 delayed-WebSocket fixture measures the current browser and unchanged server through a
bounded application-message delay. It runs two real clients in the generated match fixture and uses
native keyboard controls. Playwright's WebSocket route calls `connectToServer()` and forwards the
original text or binary value unchanged. Each direction has its own FIFO with 64-frame and 1MiB
limits, one due-time timer, and deterministic cycling jitter. Closing either endpoint cancels both
queues before propagating the close.

Run the pinned Chromium project with retries disabled:

```sh
E2E_MATCH_FIXTURE=arena-v1-matches \
  npm --prefix e2e run test:latency
```

The two profiles are unshaped local delivery and 50ms with deterministic ±10ms jitter per direction.
For each profile, ten movement/release cycles produce twenty observations. Node's monotonic clock
records the native key-call start and finish, routed outgoing movement, and the peer's first matching
authoritative snapshot as delivered to the page. The harness installs its native `WebSocket`
subclass at `document.readyState === "interactive"`: Playwright's route initialization has completed,
while the application's deferred module has not started. A real page `message` listener forwards
only sanitized public facts to a Node binding. Every received revision must map to an earlier actual
downlink dispatch from the delay queue. DOM key-event timings use the browser's separate clock and
are reported only as supporting evidence.

The first delivered snapshot also supplies one sanitized copy of the public arena dimensions,
terrain, and destructible blocks. The harness applies the authoritative 1,024-unit cell scale and
320-unit actor half-footprint to every measured position. A movement or stable witness must have no
live bomb and retain at least 128 units of unobstructed clearance in the commanded direction. This
prevents a wall or block collision from masquerading as successful release.

Before every native keydown, the harness compares east and west clearance at the latest delivered
position, selects the larger collision-free interval, and requires at least one full cell of runway.
Direction is therefore driven by current public geometry rather than a fixed alternating pattern;
variable publication timing cannot accumulate several unequal legs into the spawn-side wall. The
chosen position, direction, and both clearance values are retained in the public report.

Before the first measured keydown, both clients must display the running phase, both game canvases
must be visible, and the actor arena must hold focus. The harness then fixes one Node-clock cutoff,
the common round, and `H`, the maximum snapshot revision that has entered either downlink. Each
endpoint must publish the same two consecutive,
bomb-empty snapshots `R1 < R2`, both newer than `H`, with an unchanged actor position. Every witness
must have its own matching endpoint/round/revision chronology from post-cutoff route ingress through
delayed dispatch to page delivery. At acceptance, each endpoint's latest delivered and dispatched
snapshot is `R2`, and both raw ingress heads agree at or beyond `R2`. Newer raw FIFO frames are allowed
only while their allocated deadlines remain in the future. The public report retains the cutoff,
watermark, full selected witness facts and times, completion time, and full last ingress, dispatch,
and page heads on success and failure. A non-null record exists before startup waits and tracks the
current startup stage, overall reason, and independent endpoint reasons. Every measured key-call
start is asserted to follow readiness completion, whose timestamp is the successful predicate's
actual evaluation time.

This is one pre-input readiness baseline. It sends no command, adds no warmup input or fixed sleep,
and does not run between samples. It does not discard, substitute, or retry a slow first movement;
all twenty profile observations and every shaped observation above 250ms retain their existing
measurement and failure semantics.

Before release, two fresh page-delivered revisions must show unobstructed movement. The harness then
records the actual delayed null-command dispatch to the server. A stable pair counts only when both
revisions entered the server-to-client route after that dispatch and were subsequently delivered to
the page. The first stable timestamp is the measured upper bound and the next fresh revision records
its confirmation; this does not claim a server acknowledgement or exact physics stop instant.

The shaped profile selects a 250ms native-action-or-release to peer-observation budget and reports
every exceedance. Both clients must also agree on one public bomb and its exactly-once action ID.
The sanitized report contains message type, direction, size and timing, public room/world facts,
aggregate queue metrics, and latency samples. Traces and HAR are disabled, and raw frames are never
stored because admission frames contain private capabilities.

This is an ordered, reliable application-message-delay experiment. It does not model TCP behavior,
packet loss, reordering, bandwidth limits, or a WAN. Twenty observations describe the tested run and
do not establish a tail-latency guarantee or precise corner playability. The slice also does not add a
disconnect/resume case; reconnect correctness remains covered by the dedicated v4 recovery suite.

Early harness diagnostics are retained as invalid measurement evidence. Runs using a two-CPU CFS
quota on an eighteen-visible-core environment observed 284–300ms maxima. A later affinity-only run
reported 113.05ms, but its observer watched Playwright's pre-delay network socket rather than actual
page delivery and therefore cannot support the envelope.

The corrected retries-zero reference used Node 24.21.0, Playwright 1.63.0, pinned Playwright Chromium revision 1243,
accepted application image `sha256:b1cee9317f0d30cee36d05c9a32a7d2933a4394f08c83a324cb713f5846b13ee`,
and Linux containers restricted to CPUs 0–1 by affinity with `cpu.max` unlimited and no observed CFS
throttling at startup. All eight cases passed in 19.4 seconds. Twenty unshaped observations measured
26.95ms minimum, 39.39ms median, and 54.47ms observed p95/maximum. Twenty shaped observations
measured 133.63ms minimum, 145.82ms median, and 157.30ms observed p95/maximum, with no 250ms
exceedance. The shaped queues peaked at two frames and 7,204 bytes. These twenty observations describe
this exact profile; they remain an observed envelope rather than a tail guarantee.

## Final deployment reference

The corrected greater-clearance trajectory passes all nine latency cases on each deployment,
including all twenty mandatory observations per native profile. Compose's shaped median/max is
148.672/161.787 ms; Kubernetes is 147.674/160.793 ms. Both remain below the unchanged 250 ms
budget. The [exact-image record](../verification.md) identifies the clean application and harness
sources, shared image, failed trajectory diagnostic and complete 21-case matrices.

Browser helpers use two-core affinity with no CFS quota. Compose applies the same app affinity;
Kubernetes keeps its committed 500m CPU limit and scheduler-managed CPUs. Actual Kubernetes app
throttling remains zero across the observed window. These stated profiles do not imply identical
CPU allocation or an unmeasured capacity/tail guarantee.
