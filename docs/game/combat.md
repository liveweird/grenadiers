# Headless bombs and combat phases

The `grenadiers_core::game::State` API runs deterministic 60 Hz updates without I/O. Bomb
placement is available through `InputAction::PlaceBomb`; see [movement](movement.md) for
inputs, coordinates, footprint contact, and placement eligibility.

## Explosions and flames

The current update expires old flames, accepts ordered inputs, moves living actors, applies
swept old-fire damage, resolves survivor placements, plans/applies the complete explosion chain,
and applies final-footprint damage before deciding the round. Bombs detonate at their captured
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

## Damage and round outcomes

`Actor::alive()` reports life state. Surviving old flames test the complete movement trace,
including a stationary or starting footprint. A player moving out of fire still dies from
that contact. Dead players cannot plant a bomb in the same tick. Newly created flames test
only final footprints: they cannot retroactively burn a route traveled before they existed.
All checks require positive-area overlap, so touching a flame's boundary is safe.

Existing bombs keep their ownership, fuse, and chain behavior after their owner dies. Decide
results only after every damage phase and the complete chain: one survivor wins, zero means
`DrawReason::MutualElimination`. If both survive exactly 10,800 Running updates, the result is
`DrawReason::Timeout`. Elimination on the final update takes precedence over timeout.
`State::result()` returns `Option<RoundResult>`; `running_updates()` reports the round counter.

After finalization, actor state, world layers, the running counter, and result remain frozen.
`step` advances the absolute tick and returns stationary traces; gameplay inputs are ignored.
Input-size and clock-overflow validation still applies atomically. See
[ADR 0006](../decisions/0006-round-finalization.md).

## Current scope and checks

Public projection/events complete M2 next. Pickups and generated arenas belong to M4.
Server scheduling, gameplay transport, and PixiJS rendering begin in M3; the browser still
shows transport diagnostics. Match scoring, countdowns, and rematches follow in M5.

Run `scripts/gates.sh server` and `scripts/gates.sh coverage`. Regression fixtures cover exact
fuse/expiry/round-time boundaries, obstruction, chains, simultaneous soft-block hits, bomb-ID
permutations, flame overlap/ignition, swept damage, simultaneous deaths, result finality, and
capacity release. Changing an entity's ID cannot change blast geometry or block destruction.
