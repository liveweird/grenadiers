# Generated arena API

Status: generator-v1 is active in live classic-v2 rooms.

`core::game::generate_arena([u8; 32])` produces `GeneratedArena`. Its getters expose the
validated arena, concealed `BTreeMap<CellId, PickupKind>`, and private `GenerationRecord`.
`into_parts()` transfers those values to a trusted server/domain owner. The record contains
generator version, exact seed, and whether deterministic fallback was used. Neither it nor
concealed contents belong in a gameplay message, ordinary log, browser bundle, or bot view.
Debug redacts the seed and concealed locations; no serialization is supplied.

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
the weight-15 interval `[365,380)` after Teleporter, Kick adds `[380,395)`, and Punch adds
`[395,410)`. The all-items pool is 410; each equipment-only pool is 115. Any mask without Punch
preserves its v0.3.44 pool, draw count and exact seeded output; all 1,048,576 item subsets are
accepted. All 524,288 earlier nineteen-item masks retain their exact outputs.

The server constructs typed worlds at each actual round start and keeps the generation record
private. Layer transitions, collection, gameplay-v2 projection, and item graphics are integrated.
Default seeds use fresh OS entropy; [ADR 0014](../decisions/0014-private-round-seeds-and-live-generation.md)
defines the optional read-once operator reproduction setting and real-browser acceptance.
The preview has one round; alternating successive match rounds are integrated in M5.
