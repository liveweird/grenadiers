# Generated arena API

Live rooms use weighted generator7 with rules10, gameplay21 and multi-round matches.
Earlier generators retain their original algorithms for explicit construction and replay.

`core::game::generate_arena([u8; 32])` produces `GeneratedArena`. Its getters expose the
validated arena, concealed `BTreeMap<CellId, PickupKind>`, and private `GenerationRecord`.
`into_parts()` transfers those values to a trusted server/domain owner. The record contains
generator version, exact seed, and whether deterministic fallback was used. Neither it nor
concealed contents belong in a gameplay message, ordinary log, browser bundle, or bot view.
Debug redacts the seed and concealed locations; no serialization is supplied.

## Live construction

`ArenaConstructionProfile::WeightedLairClearanceV7` selects the weighted generator and
lair-aware spike layout together. Bomb Capacity uses the configured ×1–×10 selection weight;
new live rooms select ×5. Block density, concealment probability, mirrored sampling, fallback
and random draw order retain generator6 behavior before the clearance pass.

With lairs enabled, a deterministic pass clears the full one-cell ring around each lair and
connects it to the corner opening network. It chooses the route removing the fewest soft
blocks, then the shortest route, and reflects the result across both axes. Concealed items
are removed only where their covering blocks are cleared; other blocks and contents retain
their sampled values. The pass consumes no randomness. With lairs disabled, generator7 has
the same physical output as generator6. At ×1 it matches the unweighted clearance prototype5.

Spike layout2 excludes lair rings and preserves a usable exit from every lair. It selects
symmetric groups of four, two or zero pits on existing clear floor without carving or
resampling. Lair placement, player spawn reservations and mob collision rules are unchanged.
See [lairs](lairs.md) and [spike pits](spike-pits.md).

Construction is immutable within a room: settings changes, successive rounds and rematches
retain the selected generator/pit pair. Private replay33 binds generator7/pits2 and accepted
weighted settings. Published replay32 remains generator6/pits1; archived generators3/4 and
prototype5 remain distinct identities. Production replay recording/export remains disabled.

## Historical algorithms

Version one fixes ChaCha20 initialization, unbiased bounded percent sampling, canonical
mirrored traversal, conditional draw order, and fallback in [ADR 0011](../decisions/0011-deterministic-arena-generation.md).
Terrain and contents are symmetric under a half-turn; both spawn openings remain clear.
Capacity/range/speed kinds are assigned before play at the specified 40/40/20 weights.
The generator validates an opening placement/escape route rather than promising that any
corner placement is safe.

Run `cargo test -p grenadiers-core generation --locked` for exact seeded golden outputs,
threshold/draw-order cases, center handling, symmetry/reservations, rejection exhaustion,
invalid-map fallback, privacy checks, and simulation-level first-bomb escape survival.
Changing the algorithm or draw semantics requires a new generator version and reviewed
replay/compatibility expectations. Do not replace the golden outputs merely to hide drift.

Expanded generator v2 appends each enabled optional item to a stable weighted pool. Wall Pass adds
the weight-15 interval `[365,380)` after Teleporter, Kick adds `[380,395)`, Punch adds
`[395,410)`, Bracelet adds `[410,425)`, and Oil Barrel adds `[425,440)`. The all-items pool is 440;
each sole-item pool is 115. Oil accepts whole mirrored placements only while its lifetime eight-site
cap permits them; an over-cap draw keeps the soft blocks and makes no replacement draw. Any mask
without Oil Barrel preserves its earlier pool, draw count and exact seeded output. Every subset
of the supported item catalog is valid. Historical activation and verification details are in the
[release record](../history/configuration-activation.md#generated-arena-activation).

The server constructs typed worlds at each actual round start and keeps the generation record
private. Layer transitions, collection, checked gameplay21 projection and item graphics are integrated.
Default seeds use fresh OS entropy; [ADR 0014](../decisions/0014-private-round-seeds-and-live-generation.md)
defines the optional read-once operator reproduction setting and real-browser acceptance.
Alternating successive match rounds use fresh generated arenas within the multi-round match.
