# ADR 0025: Production journal allocation and explicit test recording

Status: accepted.

## Context

The bounded semantic journal has no production reader, export, restart-recovery or operator
consumer. Reserving its full vector in every room pays allocation and capture costs without
providing an available product capability. Recording compatibility remains valuable test evidence.

## Decision

`MatchRuntime::new` holds no journal in production or tests. Only the explicit test constructor
`new_recording` allocates a real journal. Disabled action/tick capture returns before replay-only
translation, round-marker extraction or incomplete-record work. The replay runner remains
capture-disabled; absence of recording never certifies an empty complete replay.

Keep all format/profile acceptance, actual seeds, operation ordering, incomplete/truncated errors
and exhaustive replay assertions. Fixed-profile recording remains unsupported. Do not remove
live world capture, generation records, publication, checked admission or reconnect state.

There is no environment toggle, Cargo feature, public endpoint or durable storage addition. A
future recording consumer needs an explicit bounded operational/retention policy. This refines
[ADR 0019](0019-reserved-seat-reconnect-and-replay.md)'s allocation policy without changing the
[ADR 0021](0021-gameplay-retention.md) compatibility boundary.

## Consequences and verification

Default rooms reserve zero journal vector bytes; this does not describe whole-room memory or RSS.
Explicit recorded fixtures retain the same finite record budget. Paired tests compare notices,
private state, publication revision, entropy timing and construction failures. Required exhaustive
mask acceptance and coverage floors remain unchanged. The optional
[allocation probe](../game/performance.md#replay-allocation-probe) records observations separately
from correctness; [measurement history](../history/replay-allocation.md) preserves its evidence.
