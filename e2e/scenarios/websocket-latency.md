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
