# Spike pit design

This design defines a new environmental hazard. Delivery status and priority live only in
the [backlog](../../BACKLOG.md); this document does not describe an available game feature.

## Requested behavior

A pit is traversable floor with three states:

1. **Dormant:** retracted spikes; contact by a living player or mob activates the trap.
2. **Warning:** a short, visible arming interval gives occupants time to escape.
3. **Raised:** spikes damage anything standing on the tile or entering it. After a few
   seconds, they retract and the trap returns to its dormant state.

Leaving during the warning does not cancel activation. Further contacts do not postpone
the rise or extend the raised interval. All occupants are checked; the triggering entity
has no special immunity. Dead bodies and cosmetic blood pools cannot activate a pit.

## Proposed settings and interactions

Use configurable defaults of **0.6 seconds warning** and **3 seconds raised**, with existing
damage precedence:
Shield prevents a hit, otherwise Heart absorbs one hit and grants normal damage grace.
Unprotected players and mobs die. Protection does not disarm or reset the trap.

Treat pits as an indestructible hazard layer on floor, separate from walls and soft blocks.
Bombs neither activate nor destroy them; blast propagation remains unchanged. Stopwatch does
not pause trap timers, consistent with independent environmental weather. Clouds continue
to pass over pits. Teleporter arrival activates or contacts only the destination tile.

After retraction, a surviving occupant may activate a fresh warning on the next update.
Do not raise spikes again without a new warning. New-round construction resets every pit.
Provide a lobby switch alongside the environmental features; All on/All off and Classic
must handle it consistently. Placement must preserve spawn safety and fair routes.
Prefer a small symmetric layout on already-clear floor, with no changes to existing block or
item sampling. The initial count and layout remain tuning choices; do not add a density control
without a concrete need. Disabling pits must preserve base generation and RNG progress.

## Simulation and acceptance boundaries

Use authoritative integer simulation ticks and publish the current state and remaining
warning/raised time. Resolve a warning deadline before movement on the rise update so a
character still occupying the pit at that instant cannot escape retroactively.

Use positive-area footprint overlap, including each actual ordinary or forced movement
segment and a Teleporter destination. Merely touching an outer edge is safe. No connecting
path is implied by teleportation. Timer overflow and later tick failures roll back atomically.

CPU and mob planning must reason about warning deadlines, raised intervals and the warning
their own passage would trigger. Preserve bounded planning work and exact replay identities;
do not disguise spikes as bomb fire or change archived rules and replay behavior.
Idle pits are path-dependent hazards: a planned route may trigger one and return after its
spikes rise. Reuse the simulation's phase/contact helpers in candidate-route validation.

Mob deaths must schedule exactly one lair respawn delay, just as fire deaths do. Define
newborn contact explicitly and prevent spawning onto a warning or raised pit. Existing
invisibility restrictions remain: publish pit phases without exposing an activator identity,
private position, seed, target or route.

Cover rise/retraction boundaries, crossings within one update, simultaneous occupants,
retriggering, protection, frozen occupants, forced movement, respawn, replay, disabled
generation and recipient conformance. The 3D arena and shared sheet need readable dormant,
warning and raised art with an immediate reduced-motion presentation.

## Integration boundaries

Keep the cosmetic blood-pool change independent. For pits, establish the core timing/contact
and layout model first; then add versioned settings, projection, replay and CPU support; finally
pair the renderer/lobby with coordinated live activation. Dormant internal foundations must
not silently change an existing live ruleset. Preserve closed historical contracts and identify
the new physics explicitly before enabling it.

Three pit frames and one blood-pool frame fit the five currently unassigned artist-sheet cells.
Append their stable IDs without changing existing sprite pixels or filling reserved entries.
