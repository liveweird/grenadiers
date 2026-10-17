# ADR 0011: Versioned symmetric arena generation

Status: implemented and active in generated classic-v2 network rooms.

## Decision

Generator version one uses exact `rand_chacha` 0.9.0 `ChaCha20Rng`, initialized from a
32-byte seed with stream zero and word position zero. Consume `next_u32` words. To draw an
integer in 0..99, reject words at or above 4,294,967,200, then take remainder 100. Bound each
draw to 16 words; exhaustion selects the deterministic fallback rather than retrying a map.
Pinning both algorithm and draw order makes the record reproducible across dependency changes.

Use the 15×13 classic border/even-even pillar lattice. Visit canonical 180-degree mirrored
pairs in row-major order; visit the unpaired center once. Skip walls and the six reserved
spawn/inward-neighbor cells. Draw occupancy below 65; only for occupied pairs draw item
presence below 30; only for present contents draw capacity below 40, range below 80, otherwise
speed. Mirror both occupancy and concealed kind. Never roll contents when a block breaks.

Validate terrain, opposite spawns, underlying floor connectivity, reserved cells, symmetry,
and contents beneath blocks. Validate an opening escape witness for both corners: move to one
reserved arm, plant there, retreat through spawn into the other arm outside range-two rays.
The simulation test exercises that route with a dense permitted block layout. A failed
validation or exhausted draw uses the same terrain/spawns with no blocks or contents.

## Privacy and activation

`generate_arena(seed)` returns an arena, typed concealed content, and a version/seed/fallback
record for trusted domain consumers. The record and contents have no Serde implementation;
Debug redacts the seed and reports only concealed count. Public snapshots remain independent
of generation records. Retain the private record for future replay without exposing or
ordinarily logging active seeds or unrevealed content.

Live activation follows [ADR 0014](0014-private-round-seeds-and-live-generation.md), after typed
layers, collection, protocol compatibility, rendering, and real-stack acceptance. Successive
round spawn rotation belongs to explicit round assignment; canceled countdown attempts do
not determine round parity. See [generation API](../game/generation.md) and the
[arena rules](../game/rules.md#arena-construction-and-fairness).
