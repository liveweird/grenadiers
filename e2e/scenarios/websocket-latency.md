# Ordered WebSocket delay acceptance

Run `E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:latency` against the documented
complete-match fixture. Retries and traces are disabled. The browser route forwards each original
application message to the real server unchanged, with an independent bounded FIFO in each
direction. Reports contain only message type, direction, size, timing, queue totals, and public game
facts.

## Scenario: the delay queue preserves FIFO order under cycling jitter

Fake harness time proves a deterministic jitter cycle cannot reorder frames and uses one due-time
timer. Delivered values retain their original object identity.

## Scenario: the delay queue rejects frame and byte overflow atomically

The per-direction limits of 64 frames and one mebibyte reject a frame before changing queue state.
The routed bridge treats overflow as a test failure and closes both endpoints.

## Scenario: canceling the delay queue drops pending frames and its sole timer

Closing a queue cancels its one timer, discards all pending values, and prevents any later delivery.

## Scenario: the delay queue reports allocated deadlines and late timer dispatch without changing order

Fake harness time stalls timer servicing beyond three allocated deadlines. Diagnostics retain each
enqueue time, FIFO-clamped due time, common dispatch time, and exact nonnegative timer lateness while
the original frame order remains unchanged.

## Scenario: page delivery evidence requires a prior delayed downlink dispatch

The page-side observation cannot count until the bounded route has actually forwarded the matching
public revision. A pre-dispatch or wrong-endpoint observation is rejected.

## Scenario: release evidence excludes stable snapshots queued before null dispatch

Equal positions that entered the delay queue before the null movement reached the server cannot
witness release. Only page-delivered fresh revisions whose server-route ingress follows that dispatch
can form the stable pair.

## Scenario: release evidence rejects a stable pair at a wall boundary

The harness derives horizontal clearance from the public arena walls, blocks, fixed-point cell scale,
and actor footprint. Equal positions at collision distance cannot witness a null-command stop; every
measured witness retains at least 128 units of forward clearance and has no live bomb.

## Scenario: trajectory chooses the greatest public collision-free clearance

Before each native movement, the harness derives east and west clearance from the latest delivered
public position and selects the larger interval, requiring at least one full cell of runway. Variable
publication timing can therefore change leg length without accumulating drift into a wall.

## Scenario: readiness rejects publications ingressed before its cutoff

The pre-input gate fixes one Node-clock cutoff and cannot reuse page publications whose matching
route ingress predates that cutoff.

## Scenario: readiness rejects only one fresh revision

One post-cutoff publication cannot establish a stationary renderer baseline. Each endpoint needs two
consecutive fresh revisions.

## Scenario: readiness rejects missing ingress, dispatch, wrong endpoint, and wrong round evidence

Each page witness must have its own matching endpoint, round, and revision chronology from route
ingress through delayed dispatch to page delivery.

## Scenario: readiness rejects an old matching pair when raw ingress or dispatch is ahead

A matching page pair cannot pass after a newer snapshot is due or has already been dispatched but
has not reached the page.

## Scenario: readiness accepts a caught-up pair with ordinary scheduled future frames

The latest delivered and dispatched revisions agree on the second stationary witness. Matching raw
ingress heads may be newer only while those FIFO frames retain scheduled future deadlines.

## Scenario: startup readiness diagnostics survive a cutoff capture failure

Diagnostics exist before startup waits begin and retain the current stage, reason, Node start time,
and available full-fact heads when the fixed cutoff cannot be captured.

## Scenario: readiness rejects moving or bomb-filled publication pairs

Both selected publications must retain the same actor position and an empty public bomb list; full
matching route chronology cannot make a moving or bomb-filled pair ready.

## Scenario: readiness rejects cross-endpoint publication fact mismatches

The two endpoints must select identical round, revision, tick, position, and bomb facts for each
witness, even when each endpoint independently has complete route and page chronology.

## Scenario: readiness never passes a stalled endpoint as time advances

Both endpoints independently need the complete two-publication chronology; advancing the evaluator
clock cannot make missing page evidence ready.

## Scenario: a synthetic 300ms first movement after readiness remains the first witness and an exceedance

Readiness completes before input and does not replace, filter, or retry movement evidence. A synthetic
first matching movement at 300ms remains the measured witness and exceeds the unchanged 250ms budget.

## Scenario: readiness baseline fixes cutoff, round, and maximum ingress revision

The gate records one cutoff, the common active round, and the maximum snapshot ingress revision
visible across both endpoints before it starts waiting.

## Scenario: game control diagnostics stay bounded and exclude transport secrets

Synthetic credential-bearing authentication, admission, error, and snapshot frames prove that the
Node-side recorder retains at most 256 allowlisted control events and 64 socket summaries. Six
hundred snapshots update only the socket's latest public tick and revision, so they cannot evict
admission, replacement, or close evidence. The report excludes tokens, capabilities, socket URLs,
and arbitrary error messages. The same isolated test proves the page collector survives a
`setContent` document replacement with per-document clock origins, while setup and attachment
failures produce only bounded fixed-label metadata and cannot fail the acceptance scenario.

## Scenario: native controls publish twenty unshaped local latency observations

Two real clients enter the generated open corridor. Ten native movement and release cycles produce
twenty Node-monotonic action-to-peer observations through an unshaped WebSocket route. A native
bomb command has one action ID and the same public bomb outcome on both clients. Every observation
comes from the application's real page-side WebSocket delivery and maps to a prior route dispatch.

## Scenario: native controls stay within the selected shaped latency budget

The same scenario applies 50ms with deterministic ±10ms jitter independently in both directions.
Every native action and release must reach the peer's authoritative public state within the selected
250ms budget. Two fresh moving revisions precede release. Both stable revisions must enter the route
after the delayed null command is dispatched and then reach the page; the second confirms the first
stable timestamp and is recorded separately.

Browser diagnostics also time existing allowlisted WebGL calls without invoking additional GL
work. The focused diagnostic scenario checks native call transparency, extension identity,
exception preservation, complete draw/delete method coverage, a 64-event slow-call ring, and
sanitized caller frames. This instrumentation observes the reconnect/audio scenarios and does
not change latency budgets or identify a cause until a slow call is captured.
