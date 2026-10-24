# Generated arena API

Status: this source candidate uses generator-v2 with Expanded-v5 rules, gameplay-v11, and
multi-round matches. Generator-v1 and classic-v2 are retained historical identities.

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
the weight-15 interval `[365,380)` after Teleporter, Kick adds `[380,395)`, Punch adds
`[395,410)`, Bracelet adds `[410,425)`, and Oil Barrel adds `[425,440)`. The all-items pool is 440;
each sole-item pool is 115. Oil accepts whole mirrored placements only while its lifetime eight-site
cap permits them; an over-cap draw keeps the soft blocks and makes no replacement draw. Any mask
without Oil Barrel preserves its v0.3.46 pool, draw count and exact seeded output. All 4,194,304
item subsets are accepted, and all 2,097,152 earlier masks retain their exact outputs.

The server constructs typed worlds at each actual round start and keeps the generation record
private. Layer transitions, collection, gameplay-v11 projection, and item graphics are integrated.
Default seeds use fresh OS entropy; [ADR 0014](../decisions/0014-private-round-seeds-and-live-generation.md)
defines the optional read-once operator reproduction setting and real-browser acceptance.
Alternating successive match rounds use fresh generated arenas within the multi-round match.
