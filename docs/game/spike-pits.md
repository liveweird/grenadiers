# Spike pits

Spike pits are an authoritative rules10 environmental hazard in live gameplay18 rooms. Their
availability, timing and current phase are shared room facts; open work and priority remain in
the [backlog](../../BACKLOG.md).

## Phases and damage

Each pit is traversable floor with three phases:

1. **Idle:** retracted spikes; positive-area contact by a living player or mob starts warning.
2. **Warning:** occupants can leave before the authoritative deadline.
3. **Raised:** contact damages players and kills mobs until the pit retracts.

Leaving during warning does not cancel activation. Further contact cannot postpone a deadline or
extend the raised interval. After retraction, contact on a later physical tick can start a new
cycle; a pit never rises without first passing through warning. Dead bodies, bombs and cosmetic
blood pools do not activate pits. Bombs do not destroy them or alter blast propagation, Stopwatch
does not pause their clocks, and storm clouds can pass over them.

The default warning is 36 ticks (0.6 seconds) and the default raised interval is 180 ticks
(3 seconds). The lobby accepts 6–120 warning ticks and 60–600 raised ticks. Shield prevents a
hit; otherwise Heart absorbs one hit and grants the ordinary damage grace. Existing grace also
protects normally. Protection does not disarm or reset the pit.

Deadlines are resolved before movement on the rise update. Contact uses strict positive-area
footprint overlap across actual ordinary and forced movement segments; edge-only tangency is
safe. Teleportation checks the destination without inventing a connecting path. Checked timing
or later update failure rolls back the complete simulation tick.

## Settings and layout

`environment.spikePits` contains `enabled`, `warningTicks` and `raisedTicks`. New rooms enable
the default tuning. All on and Restore enable pits; All off and Classic disable them. Disabling
retains the checked timing values for later re-enabling. Settings lock with the rest of the room
configuration when the match begins.

`spike_pit_layout_v1` selects zero, two or four immutable cells on already-clear floor. The
result is symmetric across both arena axes, row-major, outside all corner-spawn escape
reservations, lairs, Oil and other excluded overlays. Placement never carves terrain, consumes
randomness or changes block/item sampling, so enabling or disabling pits does not alter the base
generation path or RNG progress.

Every new round resets the selected pits to idle. The authoritative snapshot publishes their
sorted `cell`, `phase` and `remainingTicks`; it never publishes an activator, seed, hidden
position, target or route. See the [gameplay18 contract](protocol-v18.md).

## CPU, mobs and presentation

CPU26 forecasts path-dependent warning and raised intervals, including warnings its own route
would trigger. Route execution is guarded by the observed public pit state and retains the
existing bounded work, normal protection, movement and planting safety budgets. The live
profiles are explicitly `SPIKE_PITS_STANDARD` and `SPIKE_PITS_EASY`; archived CPU25 behavior
and historical replay identities remain closed.

Mobs do not deliberately enter pits from safe floor. A mob already overlapping a pit is allowed
to move out, including a newborn exceptional pose; pit deaths schedule the same single lair
respawn delay as other mob deaths. This escape behavior does not promise aggressive crossings.

The Three.js arena renders `terrain.spikePit.idle`, `terrain.spikePit.warning` and
`terrain.spikePit.raised` as floor-aligned billboards. Warning pulses only when reduced motion is
off; reduced motion keeps the same immediately readable static phase artwork. Blood pools remain cosmetic and do not change trap timing or contact.

The checked clock, geometry and placement APIs are documented in
[spike-pit core primitives](spike-pit-core.md). CPU and renderer checks belong to the normal
[testing workflow](../testing.md); dated acceptance evidence belongs in release/history records.
