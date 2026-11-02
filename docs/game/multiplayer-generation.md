# Multiplayer arena generation

Generator6 serves current rooms on every supported board. It reuses the v3/v4 terrain,
reflection, spawn and bounded-sampling machinery with a checked Bomb Capacity drop multiplier.
At multiplier1 it takes the exact historical sampler path; at other values it changes only the
Bomb Capacity interval before normalization. Seeded results are deterministic for the complete
configuration, including the multiplier. A changed sampling bound can change rejection draws
and later cells, so equal seeds across multipliers need not yield identical layouts.

Historical generator v3 serves three- and four-player 15×13 profiles and remains available for
headless two-player compatibility. `generate_multiplayer_arena` accepts
a private 32-byte seed, an `EnabledItems` mask, and `ArenaGenerationConfig` containing the player
count, board size, and actual-round assignment. Its historical profile explicitly rejects
11×9 and 13×11; those historical presets use generator4 through the [board geometry](board-sizes.md)
profile. Older two-player 15×13 profiles retain generator2.

Version three traverses eligible classic-floor cells by row-major orbit representative. Each
orbit is deduplicated under horizontal and vertical reflection and therefore contains one, two,
or four cells. A single block decision and, when applicable, a single concealed-item decision
apply to the whole orbit. Oil Barrel is accepted only when its complete orbit fits the eight-site
lifetime cap; an over-cap choice retains the blocks and performs no replacement draw.

All four corner spawns and both inward cells of each corner L remain clear even in three-player
rounds. Four-player arenas supply corners in row-major order. Three-player arenas omit corner
`((round - 1) + 3) % 4` in top-left, top-right, bottom-left, bottom-right order; the existing
`RoundAssignment` then rotates the three participating seats across the remaining sorted spawns.
Two-player v3 arenas retain the opposite top-left/bottom-right pair. Historical generator v1/v2
entry points and seeded outputs remain unchanged.

If bounded rejection sampling or arena validation fails, v3 returns the same round's spawn count
and corner selection on valid classic terrain with no blocks or concealed items, and marks the
compact `GenerationRecord` fallback flag. The record retains only version, seed, and fallback;
private replay headers record immutable board/count configuration separately from this compact record.

Run `cargo test -p grenadiers-core --locked generation::generated_v3` for deterministic goldens,
reflection and escape reservations, oil capacity, bounded fallback, board rejection, and round
assignment coverage.
