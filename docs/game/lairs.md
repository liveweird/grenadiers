# Lair and mob simulation

Live rules10/gameplay19 gives each lair exactly one living mob. A lair waits for its mob to die,
then starts a fresh respawn delay. See [the wire contract](protocol-v18.md) and
[CPU boundaries](multiplayer-cpu.md#lair-profile-boundaries).

## Configuration and lifecycle

`RulesConfig::environment_v10(settings, clouds, RespawnLairConfig, SpikePitConfig)` accepts validated
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
collision and committed retreat anchors prevent wall crossing. Navigation3 can cross multiple
future blast cells to reach refuge before stationary timed bombs detonate. It accounts for whole
mob footprints, fuse deadlines and chain ignition rather than requiring the first neighbouring
cell to be outside every future blast. Turns remain aligned to cell centers and changed hazards
or obstructions are rechecked.

The temporal search shares a bounded work allowance. Concealed/Pending Oil uses conservative
potential blast geometry; Pending cells cannot be used as transit or refuge. Unrelated flames
and Burning Oil remain local obstacles. Ambiguous moving, Remote, Contact or fire-triggered bombs,
arithmetic failure and work exhaustion retain navigation2 avoidance. Conservative forecasts may
refuse some otherwise possible routes; no future item-reveal simulation is required.

Under rules10, ordinary mob routing treats spike-pit cells as unavailable. A mob already
overlapping a pit may move outward, but this escape exception does not authorize deliberate
crossings. Raised-pit death schedules exactly one normal replacement delay.

Mobs do not block players or blasts, plant bombs, collect items, use equipment, drop rewards or
affect scores. Swept player/mob contact uses Shield, Heart/grace and elimination rules.
Bracelet travel counts before actions and pickups; teleportation checks destinations without
a connecting sweep. Flames, blasts and burning Oil kill mobs; movement can trigger mines.

Stopwatch freezes mobs while respawn delays advance. Frozen mobs remain vulnerable and
hazardous on contact. Thawing adds no movement credit. Round results freeze simulation.
Checked timer/ID failures restore the complete tick, including simulation randomness.

## Observations and compatibility

`State::lair_statuses()` exposes rules10 cells, living counts and optional absolute deadlines:
`None` while occupied, `Some(deadline)` while waiting. Gameplay18/19 publish the corresponding
`spawnRemainingTicks`: null while occupied, otherwise 0–delay. Zero means eligible but possibly
obstructed. `State::mobs()` omits pursuit targets, paths and private thaw deadlines.

Rules8 retains its periodic 1–8-mob lifecycle and finite `State::lairs()` view for historical
contracts/replay7. Rules9 and rules10 use the separate view. Mob AI has its own immutable identity: all existing rules
constructors and replay8 retain navigation1; replay9/10 retain navigation2. Live construction
explicitly selects navigation3; private replay11 binds CPU26 and replay12 binds CPU27 under gameplay18. Replay13 binds
gameplay19 with CPU27 on Easy/Standard and CPU28 on Hard; replay14 retains navigation3
and advances Hard to CPU29. Replay15 retains navigation3 and binds Easy/Standard31 or Hard32. Replay16 retains navigation3 and binds
Easy/Standard33 or Hard34 with certified one-update pickup handoffs. Replay17 retains navigation3 and binds
Easy/Standard33 or Hard35. Replay18 retains navigation3 and binds Easy/Standard36 or Hard37. Replay19 binds Easy/Standard38 or Hard39 with the same navigation3 identity.
Settings changes, rounds and rematches
preserve that selection. These policies share the same speed, collision/contact and recipient
facts; gameplay18/19 carry no hidden navigation data. Production replay recording remains disabled.

The 3D arena retains the dedicated `terrain.lair` doorway sprite upright on the front of a
taller stone lair, with `material.lair.top` and `material.lair.side` supplying its surface maps.
It replaces the ordinary pillar visual at that cell without changing the indestructible obstacle.

Run `cargo test -p grenadiers-core --locked lair --lib` and repeat with `--release` for focused
lifecycle, movement, death, obstruction, allocation and rollback regressions.
