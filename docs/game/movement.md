# Headless world and movement

M1 provides `grenadiers_core::game`, a pure Rust domain API. It can run without a server,
browser, database, graphics library, or clock. The diagnostic UI and wire protocol are
unchanged. See [ADR 0002](../decisions/0002-deterministic-movement.md) for the numeric and
turning decisions and the crate's runnable documentation example for state construction.

## State and spatial layers

`Arena` validates a 15×13 terrain array, separate soft blocks, and two opposite-corner spawns.
Spawns and their inward neighbors must be clear; the underlying floor must remain connected
when blocks are ignored. `fixed_arena()` supplies the symmetric hand-authored preview fixture,
including three-cell opening routes. It is not the future random map generator.

`World` keeps the arena, owned bombs, pickup cells, and hazard cells separate. `State::new`
validates two `ActorInit` values and canonicalizes their order by seat. Their IDs must be
unique and their full footprints legal. Actors may share positions or pass through one another.
Read-only accessors expose domain state; this internal model is not a public network snapshot.

Coordinates increase east and south from the top-left. `Position` uses 1,024 units per cell;
`Position::center` gives a cell center and `Position::cell` checks coordinate conversion.
An exact cell boundary belongs to the cell east/south of it. Footprints may touch an obstacle
face but cannot overlap its interior.

## Advancing one tick

Call `State::step(&[SeatInput])` with already-accepted logical `Press`, `Release`, `Clear`, and `PlaceBomb`
events in authoritative order. New presses take priority; repeat presses do not. Releasing
the winner resumes the last held direction. Clearing or releasing the last direction stops
movement and clears its fraction immediately, including within a multi-event tick batch.

Each successful call advances one absolute 60 Hz tick; active rounds also count Running updates. All five speed levels retain fractional travel
between updates. There is no accumulated movement burst after hitting a wall. A perpendicular
turn needs an offset within the configured 64–256 units (1/16–¼ tile; default 256) and a fully
clear correction path. Classic-v3 fixes this at 256; historical classic-v1/v2 retain 128 units
(⅛ tile). Alignment consumes
the normal distance budget and can span ticks; forward travel starts after alignment completes.
The last committed heading remains authoritative while a correction is pending or rejected.

`StepReport` contains the completed tick, two movement traces in seat order, and phase events. Each trace
retains the starting footprint and up to two actual cardinal segments, including turning
correction. `overlaps_cell` uses positive-area contact for swept damage against existing fire.
It must not replace the piecewise path with a diagonal bounding rectangle.

More than 64 inputs or tick overflow rejects the whole step without mutating state. The
[room transport](rooms.md) supplies authentication, sequencing, expiry and bounded admission.

## Bomb placement and collision

`InputAction::PlaceBomb` requests one bomb in the actor's cell after movement. Each actor
starts with capacity one and range two. A bomb captures owner, range, placement tick, and
the configured fuse deadline (180 ticks by default). `World::bombs()` exposes complete records;
`live_bomb_count(seat)` derives ownership counts. Domain records expose an explicit trigger;
`detonates_at()` returns a deadline only for Timed bombs. Current rules reject Remote/Contact
before simulation, so no untimed bomb can enter a live Classic world. Fuses and flames now execute; [combat phases](combat.md) describe
their timing, damage ordering, results, and immutable occupancy rules.

Accepted manual requests remain separate in bounded per-seat queues; raw-core `Clear`
cancels earlier requests for its seat in the same batch, while later requests survive.
The Classic placement adapter derives one simultaneous claim per seat from those queues,
so repeated requests never retry a failed placement or place extra bombs in one step. Placement
requires capacity and an unblocked floor cell without an existing bomb.
Two eligible requests for one cell both fail; an ineligible request cannot deny its opponent.
Distinct placements commit in seat order. Failure consumes neither an ID nor capacity.

Every overlapping actor receives an exemption and can leave. Exact boundary clearance
revokes that exemption before any further segment; re-entry or crossing another bomb is blocked.
The earlier `add_bomb_obstacle` API is retired; collision fixtures use test-only owned bombs.
Capacity, range and speed pickups follow the implemented generated-round upgrade rules. Hazards
carry exclusive flame expiry ticks and can ignite
bombs and damage actors through the documented combat phases. See [ADR 0004](../decisions/0004-bomb-placement.md)
for placement ordering, identity allocation, and compatibility decisions.

## Verification and next integration

Run `cargo test -p grenadiers-core --locked` for arena, input, geometry, movement, state, and
determinism tests, plus the public API example. The workspace gate and coverage floor still
apply: `scripts/gates.sh server` and `scripts/gates.sh coverage`.

M2 adds combat, explicit public projections, and phase events through the same state transition. M3 will introduce
versioned public gameplay contracts and [PixiJS rendering](../decisions/0003-arena-rendering.md).
Identical input logs must reproduce complete states independently of browser frame rate.
