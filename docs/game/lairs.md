# Lair and mob simulation

Live rules9/gameplay17 gives each lair exactly one living mob. A lair waits for its mob to die,
then starts a fresh respawn delay. See [the wire contract](protocol-v17.md) and
[CPU boundaries](multiplayer-cpu.md#lair-profile-boundaries).

## Configuration and lifecycle

`RulesConfig::environment_v9(settings, clouds, RespawnLairConfig)` accepts validated
`RespawnLairSettings::new(spawn_seconds)`: 5–60 seconds, default ten. Capacity is fixed at one;
the lobby edits only **Respawn delay**. Disabled tuning retains its delay for re-enabling.

Each empty lair first becomes eligible one delay after round start. Successful spawning suspends
that lair's timer while its mob lives. Death at tick D schedules replacement at D + delay,
independently of the other lair. Repeated fire processing cannot restart this cooldown.
An obstructed eligible spawn retries next tick; it neither adds another delay nor accumulates
births. Spawn-direction priority rotates only after a successful birth.

Spawn cells must be adjacent clear floor outside living actor/mob footprints, bombs, fire and
every Oil state. At most two mobs exist across the two lairs. IDs use the checked world
allocator; newborns first move on the following tick.

The indestructible pillars use `lair_layout_v1(board)`: `(6,6)/(8,6)` on 15×13,
`(6,4)/(6,6)` on 13×11 and `(4,4)/(6,4)` on 11×9. Generator and randomness versions are
unchanged. Construction rejects a lair cell without a hard wall.

## Movement and combat

Mobs move at 1.5 cells/second and chase the nearest reachable living, uncloaked player,
using round priority for ties. Shared blast geometry guides bomb avoidance. Fixed-point
collision and committed retreat anchors prevent wall crossing and direction oscillation.

Mobs do not block players or blasts, plant bombs, collect items, use equipment, drop rewards or
affect scores. Swept player/mob contact uses Shield, Heart/grace and elimination rules.
Bracelet travel counts before actions and pickups; teleportation checks destinations without
a connecting sweep. Flames, blasts and burning Oil kill mobs; movement can trigger mines.

Stopwatch freezes mobs while respawn delays advance. Frozen mobs remain vulnerable and
hazardous on contact. Thawing adds no movement credit. Round results freeze simulation.
Checked timer/ID failures restore the complete tick, including simulation randomness.

## Observations and compatibility

`State::lair_statuses()` exposes rules9 cells, living counts and optional absolute deadlines:
`None` while occupied, `Some(deadline)` while waiting. Gameplay17 publishes the corresponding
`spawnRemainingTicks`: null while occupied, otherwise 0–delay. Zero means eligible but possibly
obstructed. `State::mobs()` omits pursuit targets, paths and private thaw deadlines.

Rules8 retains its periodic 1–8-mob lifecycle and finite `State::lairs()` view for historical
contracts/replay7. Rules9 uses the separate view and test-only replay8; production replay
recording remains disabled. The artist sheet and mob/lair sprites are unchanged.

Run `cargo test -p grenadiers-core --locked lair --lib` and repeat with `--release` for focused
lifecycle, movement, death, obstruction, allocation and rollback regressions.
