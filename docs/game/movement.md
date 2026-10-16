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

`World` keeps the arena, bomb obstacles, pickup cells, and hazard cells separate. `State::new`
validates two `ActorInit` values and canonicalizes their order by seat. Their IDs must be
unique and their full footprints legal. Actors may share positions or pass through one another.
Read-only accessors expose domain state; this internal model is not a public network snapshot.

Coordinates increase east and south from the top-left. `Position` uses 1,024 units per cell;
`Position::center` gives a cell center and `Position::cell` checks coordinate conversion.
An exact cell boundary belongs to the cell east/south of it. Footprints may touch an obstacle
face but cannot overlap its interior.

## Advancing one tick

Call `State::step(&[SeatInput])` with already-accepted logical `Press`, `Release`, and `Clear`
events in authoritative order. New presses take priority; repeat presses do not. Releasing
the winner resumes the last held direction. Clearing or releasing the last direction stops
movement and clears its fraction immediately, including within a multi-event tick batch.

Each successful call advances one 60 Hz tick. All five speed levels retain fractional travel
between updates. There is no accumulated movement burst after hitting a wall. A perpendicular
turn needs an offset of at most 128 units and a fully clear correction path. Alignment consumes
the normal distance budget and can span ticks; forward travel starts after alignment completes.
The last committed heading remains authoritative while a correction is pending or rejected.

`StepReport` contains the completed tick and two movement traces in seat order. Each trace
retains the starting footprint and up to two actual cardinal segments, including turning
correction. `overlaps_cell` uses positive-area contact and supports M2's later hazard checks.
It must not replace the piecewise path with a diagonal bounding rectangle.

More than 64 inputs or tick overflow rejects the whole step without mutating state. The
transport layer will still need authentication, sequencing, expiry, and queue/rate limits.

## Bomb collision interface and scope

`State::add_bomb_obstacle` inserts a validated collision record with a unique entity ID.
Every overlapping actor receives its own exemption and can leave. Exact boundary clearance
revokes that exemption before any further segment; re-entry or crossing another bomb is blocked.
This method does not implement player bomb placement, capacity, fuses, explosions, or damage.
Pickup and hazard layers do not yet apply effects. Those mechanics belong to M2/M4.

## Verification and next integration

Run `cargo test -p grenadiers-core --locked` for arena, input, geometry, movement, state, and
determinism tests, plus the public API example. The workspace gate and coverage floor still
apply: `scripts/gates.sh server` and `scripts/gates.sh coverage`.

M2 will consume the traces for combat and extend the same state transition. M3 will introduce
versioned public gameplay contracts and [PixiJS rendering](../decisions/0003-arena-rendering.md).
Identical input logs must reproduce complete states independently of browser frame rate.
