# ADR 0013: Gameplay v2 and immutable room profiles

Status: implemented; generated live-room activation remains separate.

## Decision

Evolve gameplay authentication and closed DTOs together to protocol v2. Diagnostics remain
v1. Require exposed pickup arrays and collection events; support only explicit rules versions
1/2 and map identifiers fixed_v1/generated_v1. Concealed/pending layers, seed, generation record,
and assignment remain private. New clients accept bounded incompatible-version responses from
old servers; wrong versions are rejected before credential lookup.

A room profile owns both actual construction and advertised rules/map labels. Never infer
map identity from geometry or silently relabel arbitrary state. Publication checks projected
rules against its immutable profile. Initially retain FixedClassicV1; generated-v2 activation
requires browser rendering, private seed handling, and real-browser pickup acceptance.

## Bounds and verification

Preserve three reports, 64 events, and 65,536 encoded UTF-8 bytes per snapshot. A classic-v2
update emits at most two placements, 16 detonations, two combined elimination/collection
actor events, and one result: at most 21 events, or 63 across three updates. No per-cell reveal,
burn, or flame events are added. Full state remains authoritative.

The maximal structural fixture includes 195 longest-kind pickups and 64 events, encoding to
44,466 bytes. Both languages validate missing/extra fields, bounds, enum values, and old/future
versions. Projection additionally checks ordering, occupancy, visibility, and profile agreement.
Upgrade both browser and server together; in-memory rounds do not survive deployment restart.
