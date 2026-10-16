# ADR 0002: Integer world and movement simulation

Status: accepted for M1. Gameplay transport and rendering integration follow in M3.

## Boundaries and state

Add a pure `grenadiers_core::game` module, preserving diagnostic transport exports. The
domain depends on neither Tokio nor browser, network, database, or wall-clock APIs.
Terrain, soft blocks, actors, bomb obstacles, pickups, and hazards occupy separate layers.
Pickup/hazard cells are spatial data only in M1; combat and collection are later milestones.

Use stable typed cell/entity/seat/tick identifiers. Two actors are canonicalized by seat;
cells sort in row-major order and entity collections use ordered keys. Equality includes
all authoritative state: input priority, fractional distance, pending turns, and bomb grace.
Rules configuration selects the immutable `classic_v1` profile. It is independent of the
diagnostic WebSocket protocol version; domain state is not a wire snapshot.

The fixed 15×13 arena validates its border, opposite-corner floor spawns, reserved inward
neighbors, block placement, and floor connectivity ignoring destructible blocks. The
hand-authored symmetric fixture has a clear opening route for both seats. Generic generated-map
blast-escape validation belongs to M4, after M2 supplies combat geometry.

## Arithmetic and tick boundary

One cell is **1,024 integer units**. The square footprint has half-width **320** units
(side 0.625 cell); the inclusive turn window is **128** units (0.125 cell).
The five speed levels are 3, 3.5, 4, 4.5, and 5 cells/second, represented by
`3072, 3584, 4096, 4608, 5120` units/second at **60 updates/second**.

For active movement, compute `total = residual + speed`, distance `total / 60`, and next
residual `total % 60`. Reversal preserves the fractional residual. Clearing input or releasing
the last held direction clears it immediately, even if a later input in that batch resumes motion.
Discard blocked whole-unit distance rather than accumulating a later movement burst.

`State::step` takes an ordered batch of already-accepted logical seat inputs. Reject more
than 64 inputs or tick overflow before any mutation. Duplicate held presses do not change
priority; releasing the winner resumes the previous held direction. This input bound is a
domain safeguard, not a substitute for future transport admission and rate limits.

## Turning and swept collision

A perpendicular request is admitted only within the inclusive turn window and when the
entire correction to the nearest cell centerline is clear. Otherwise it produces no movement
and does not change the committed heading. Straight motion and reversal do not start alignment.

Alignment consumes the same distance budget as forward travel and can span several ticks.
Store its direction/target separately; commit the new heading only when alignment finishes.
Any winning-input change or clear cancels pending alignment. Resolve replacement input against
the last committed heading; stopping and resuming re-evaluates the remaining correction.
This avoids both diagonal speed bonuses and bypassing the window by holding a rejected turn.

Sweep each cardinal segment against terrain, blocks, and non-exempt bomb cells. Clamp to the
nearest obstacle face; boundary contact is allowed, positive-area penetration is not. Players
do not block each other. Retain the actual piecewise path, at most two segments per tick,
for M2 swept hazard tests; an enclosing diagonal rectangle would report false contacts.

Adding a bomb obstacle grants a separate exemption to every overlapping actor. Revoke it
as soon as an actual segment fully clears that cell, including exact edge contact, before
processing another segment. Hypothetical correction checks must not mutate actual exemptions.
This interface models collision only; M2 will implement placement eligibility, fuses, and damage.

## Evidence

Tests must cover all speed levels, obstacle boundaries, maximum-speed sweeps, partial turns,
rejected/reversed/cancelled input, overlapping actors, bomb exit/re-entry, trace geometry,
atomic errors, and repeated input logs producing identical complete states. No renderer or
network service is needed to run the domain suite.
