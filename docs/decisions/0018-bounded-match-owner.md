# ADR 0018: Bounded match ownership and departure boundaries

Status: dormant owner implemented; coordinated production activation follows.

## Decision

Compose MatchRuntime with one bounded owner task. Preserve 32 command slots, two bindings,
two pending departures/replies, four-update catch-up, and the 500 ms lag limit. Validate full
room/epoch/binding identity before input mutation. Fence accepted departures immediately but
retain occupancy until one scheduled batch, preventing intermediate replacement or rematch.

Commit Running physics/scoring before the departure batch. In every idle phase apply the
batch before automatic transition, using the last completed tick. This avoids consuming a
new round/seed after an accepted countdown-boundary departure without adding a synthetic tick.

Retain creation-only prematch expiry, a fresh lobby deadline after rematch, and fixed
retirement only after match finality. Continue the global clock while rematch is available.
Use sticky shutdown independent of command capacity. Atomically interrupt then apply pending
departures and publish once; keep pending reports and final outcomes. Closing watch values
carry the last checked publication so close cannot overwrite state before a slow reader sees it.

## Process boundary

Activation must signal owners and sockets, then explicitly drain upgraded WebSocket tasks
within a deadline. Axum's HTTP graceful shutdown alone does not await detached upgrade
callbacks. Preserve existing credential, sequence, generation, and per-send timeout ownership.
