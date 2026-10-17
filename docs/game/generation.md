# Generated arena API

Status: explicit headless generator groundwork. Live rooms continue using `fixed_arena()`.

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

The API does not place typed contents into an active world yet. Layer transitions, pickup
collection/upgrades, the wire contract, item graphics, and server activation ship separately.
