# ADR 0007: Explicit public state and phase events

Status: accepted for the final M2 increment.

## Decision

Expose `State::public_view()` as an owned `PublicState` projection. Copy visible arena terrain
and blocks, seat-ordered actor data, owned bomb properties, flame expiry ticks, remaining round
time, and result. Use explicit `ArenaView`, `ActorView`, `BombView`, and `FlameView` types.

Exclude held-input priority, fractional travel, pending turns, collision exemptions, entity
allocation, and future concealed item data. Do not serialize `State` or `World` directly.
The domain projection has no Serde or network dependency; M3 will define and validate wire DTOs
separately. Future private world fields therefore do not automatically become public payloads.

`StepReport::events` contains `StepEvent { tick, kind }` records emitted by the actual successful
phase. Record placements, detonations, actor eliminations, and round end. Placement/damage use
canonical seat order; detonations use bomb-ID order after planning the complete chain. Old-fire
eliminations precede placement; new-fire eliminations follow detonation; round end is last.

Keep the complete `BombView` on placement and detonation events. A bomb placed and detonated
within one update remains observable even though it is absent from the final snapshot. Never
infer events only by comparing before/after world maps. Test fixture insertion emits no events,
failed placement emits none, and finalized rounds emit no repeated gameplay events.

## Verification

Verify projected map layers, actors, ownership counts, deadlines, timer, and outcomes against
state transitions. Changing only private bookkeeping must leave the public view unchanged.
Exercise same-tick placement/explosion/death/result ordering, old-fire death before survivor
placement, canonical input/actor ordering, and empty reports after finalization. Existing
movement, collision, combat, and atomic-error regressions must remain unchanged in behavior.
