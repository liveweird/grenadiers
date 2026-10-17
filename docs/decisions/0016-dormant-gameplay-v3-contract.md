# ADR 0016: Dormant gameplay-v3 match contract

Status: structural schema/DTOs implemented; checked projection and production activation follow.

## Decision

Stage v3 alongside active v2 rather than populate match fields from the single-round owner.
Add self-contained `api/gameplay-v3.schema.json`, `core::protocol::v3`, and generated
`web/src/game/v3/schema.ts`. No alternate public endpoint or feature flag is introduced.
Each version has its own authenticated version value. The active endpoint remains v2;
diagnostics remain v1. Browser and server switch together only after real match adapters pass.

Keep physical game state, events, counters, and identity definitions identical to v2, with
explicit parity tests. The existing generator supports local references, so the v3 schema
intentionally repeats unchanged definitions. Both schema/output pairs participate in generation
and drift checks. ESLint/Knip exceptions apply only to exact generated files.

## Match facts

V3 rooms require bounded scores, actual round count, nullable latest completed round, and
per-seat rematch readiness. Completed rounds retain epoch, actual number, end tick, and result;
match winner/forfeit/abandonment/interruption is independent. Command epochs remain monotonic
across cancellation, rounds, and rematches. Actual round count is zero before first start and
after rematch; an inter-round countdown retains the preceding actual count.

Countdowns distinguish initial/inter-round. Nonfinal result holds advertise the next countdown
tick. Final matches advertise end time and rematch eligibility after 180 authoritative ticks.
Prematch interruption has no fabricated match outcome. A sequenced, epoch-fenced rematch
command sets or withdraws consent. Cross-field truthfulness belongs in checked projection,
not an invented single-round approximation.

## Bounds

Preserve closed required objects, scalar bounds, three reports, 64 events, and 65,536 UTF-8
bytes. A worst-metadata fixture with 195 longest pickup entries and 64 largest physical events
encodes to 44,742 bytes, leaving 20,794 bytes. Unsupported-version responses accept 0..65535
so clients understand older/future peers. No extra physics version or gameplay event is added.
