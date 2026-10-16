# Headless bombs and combat phases

The `grenadiers_core::game::State` API runs deterministic 60 Hz updates without I/O. Bomb
placement is available through `InputAction::PlaceBomb`; see [movement](movement.md) for
inputs, coordinates, footprint contact, and placement eligibility.

## Explosions and flames

The current update expires old flames, accepts ordered inputs, moves actors, resolves player
placements, and plans/applies the complete explosion chain. Bombs detonate at their captured
deadline (placement tick plus 180) or when their cell contains an active flame. A newly placed
bomb can join that same tick's chain.

Each explosion burns its center and four cardinal rays up to its captured range. Hard walls
and arena boundaries stop rays without burning. Soft blocks burn and stop a ray. Bombs burn,
join the queue, and stop a ray. Every ray in the complete chain observes the same post-placement
bomb/block occupancy, including objects already marked for removal. Apply removals only after
planning finishes. See [ADR 0005](../decisions/0005-explosion-snapshot.md).

`World::hazards()` returns a sorted cell-to-exclusive-expiry map. A flame created at tick `t`
remains active through `t + 29`; expiry happens before movement at `t + 30`. Overlapping fire
keeps the later expiry. Lingering flames do not propagate new rays without another bomb.
At the terminal representable tick expiry saturates; advancing beyond that tick is an error.

Detonation removes the owned bomb and every actor's associated collision exemption. Capacity
is derived from remaining bombs. A slot freed by an explosion is usable on the next tick,
since this tick's placement has already been resolved.

## Current scope and checks

Flames currently ignite bombs and destroy soft blocks, but do not yet kill actors. Damage,
sole-survivor/draw finalization, the 10,800-update timeout, and public projection/events are the
remaining M2 slices. Pickups and generated arenas belong to M4. Server scheduling, gameplay
transport, and PixiJS rendering begin in M3; the browser still shows transport diagnostics.

Run `scripts/gates.sh server` and `scripts/gates.sh coverage`. Regression fixtures cover exact
fuse/expiry ticks, obstruction, chains, simultaneous soft-block hits, bomb-ID permutations,
flame overlap/ignition, grace cleanup, and capacity release. Changing an entity's ID cannot
change blast geometry or block destruction.
