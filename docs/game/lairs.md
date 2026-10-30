# Lair and mob simulation

Rules8 supplies a headless lair overlay. The live gameplay16 endpoint constructs rules8;
its wire messages, CPU profiles and rendering remain governed by that contract. Delivery status
lives in the [backlog](../../BACKLOG.md).

## Configuration and layout

Construct `RulesConfig::environment_v8(settings, clouds, lairs)` with an explicit `LairConfig`.
`LairSettings::new(spawn_seconds, mobs_per_lair)` accepts 5–60 seconds and 1–8 living mobs
per lair; defaults are ten seconds and three mobs. Old rules constructors disable lairs.

Two indestructible wall pillars carry the overlay. `lair_layout_v1(board)` selects `(6,6)/(8,6)`
on 15×13, `(6,4)/(6,6)` on 13×11 and `(4,4)/(6,4)` on 11×9. Arena-generation outputs and
random streams are unchanged. State construction rejects a lair cell without a hard wall.

Each lair first attempts spawning after one interval. Full or obstructed attempts wait another
complete interval without accumulating a burst. Spawn cells must be adjacent clear floor,
outside actor/mob footprints, bombs, fire and every Oil state. Death releases capacity. At most
16 mobs exist; IDs share the checked world allocator. Newborns first move on the following tick.

## Movement and combat

Mobs have a 1.5-cell/second movement budget and pursue the nearest reachable living, uncloaked
player, using round-priority ties. Shared blast geometry guides bomb avoidance; at most four
board-sized searches run per update. Fixed-point collision and committed retreat anchors prevent
wall crossing and direction oscillation. Pending Oil waits for mob footprints to clear.

Mobs do not block players or blasts, plant bombs, collect items, use equipment, drop rewards or
affect scores. Swept player/mob contact uses the existing Shield, Heart/grace and elimination
rules. Bracelet travel counts before actions and pickups; teleportation checks destinations
without a connecting sweep. Flames, blasts and burning Oil kill mobs; movement can trigger mines.

Stopwatch freezes existing and newly spawned mobs while lair timers continue. Frozen mobs remain
vulnerable and hazardous on contact. Thawing adds no stored movement credit. Results freeze
motion and timers. Checked timer/ID failures roll back the complete simulation and randomness.

## Headless consumers and checks

`State::lairs()` exposes cells, absolute next-attempt ticks and living counts. `State::mobs()`
exposes IDs, lair indices, positions, headings and a frozen boolean, without pursuit targets or
Stopwatch deadlines. These getters do not change existing public-state or actor DTO layouts.

The [gameplay16 contract](protocol-v16.md) checks canonical overlays and recipient privacy.
Its [CPU20 boundary](multiplayer-cpu.md#lair-profile-boundaries) adds finite NPC/spawn contact
proofs; the explicit roster runtime and private replay7 regenerate the same simulation from
accepted inputs. The artist sheet includes `terrain.lair` and four `mob.<direction>` poses;
see the [artist handoff](../../assets/art/README.md).

Run `cargo test --locked -p grenadiers-core lair --lib` and repeat with `--release` for focused
movement, layout, interaction and rollback regressions.
