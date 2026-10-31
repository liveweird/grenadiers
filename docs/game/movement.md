# World and movement

`grenadiers_core::game` is a deterministic Rust simulation independent of browsers, networking,
storage and wall clocks. Live rules10 supports two to four players on the three
[board presets](board-sizes.md). [ADR 0002](../decisions/0002-deterministic-movement.md)
explains fixed-point coordinates and turning.

## State and spatial layers

`Arena` separates hard terrain, soft blocks and spawn metadata. `World` adds owned bombs,
pickups, Oil and hazards; `State` adds actors and round/environment state. Construction
canonicalizes actors by seat and validates unique IDs, legal footprints and rules10 player
occupancy. Read-only domain accessors are separate from recipient-filtered wire snapshots.

Coordinates increase east and south. Each tile spans 1,024 units; an actor footprint is
640×640 units. `Position::center` and `Position::cell` handle tile conversion. Exact boundaries
belong to the cell east/south of the boundary. Footprint edge-touching is legal.

## Player blocking

Living players cannot share a center tile or overlap footprints. Humans, CPUs, cloaked and
frozen players all block; dead players do not. Wall Pass crosses soft blocks, never players.

Each movement phase reserves every living player's starting tile and footprint. Existing round
priority resolves contenders. Accepted piecewise movement reserves its entered tiles and swept
footprints for the rest of that phase. Head-on approaches stop safely; ordinary swaps, crossing
paths and entry into a tile vacated this tick wait until the next tick. Priority decides which
player may enter an otherwise empty contested tile.

Movement clips before committing positions, turn state, collision grace, kicks or damage traces.
Bracelet uses physical blocking-face contact, including zero accepted movement. A tile-only
reservation without physical contact does not shove. Forced travel respects starting cells,
voluntary sweeps and earlier accepted shoves. A fully blocked shove consumes no cooldown.
Teleporter exchanges are validated atomically against other players without a connecting sweep.

Rules1–8 retain their historical player pass-through behavior and input replay semantics.

## Advancing one tick

`State::step(&[SeatInput])` accepts logical Press, Release, Clear and manual-action inputs.
New presses take priority; repeat presses do not. Releasing the winning direction resumes the
last held direction. Clearing/releasing the last direction stops movement and its fraction.

Each call advances one absolute 60 Hz tick; active rounds also count Running updates.
Five speed levels retain fractional travel. Blocked movement accumulates no burst. Perpendicular
turns require an offset within configured tolerance (64–256 units, default256) and a clear
alignment path. Alignment consumes the same movement budget, can span updates, and precedes
forward travel. The last committed heading remains authoritative during correction.

`StepReport` records one trace per participating seat and phase events. Each trace contains
its start and up to two accepted cardinal segments, including alignment. Swept fire/contact
checks use that piecewise path rather than a diagonal bounding rectangle.

Input-count/tick overflow fails before mutation. Checked late pickup/lair failures restore the
whole tick, including randomness. The [room transport](rooms.md) owns authentication,
sequencing, input expiry and bounded admission.

## Bomb collision and verification

Bomb placement occurs after movement and captures owner, range, placement tick and fuse/trigger.
[Item rules](expanded-items.md) determine special weapons and timed effects. Placement requires
capacity and legal free floor; failed placement consumes neither an ID nor capacity.

Actual placement grants overlapping actors exit grace. Full footprint exit permanently removes
that exemption; re-entry and unrelated bombs remain blocking. Turn correction and forward travel
use the same obstacle and player-collision rules. [Combat phases](combat.md) define explosions,
damage and immutable results.

Run `cargo test -p grenadiers-core --locked player_collision` and repeat with `--release` for
focused player/Bracelet/Teleporter cases. Full server and coverage gates remain required.
