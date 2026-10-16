# ADR 0006: Damage phases and immutable one-round outcomes

Status: accepted for the third M2 increment.

## Decision

Track actor life and the number of Running updates independently of the absolute simulation
tick. Resolve one round without introducing match scores, countdowns, rematches, or reconnect.

Expire flames before movement. Move living actors and test their complete piecewise traces,
including their starting footprints, against surviving old fire. Positive-area contact kills
the actor before bomb placement. Dead actors cannot move or place bombs. Their previously
placed bombs keep their normal fuse, obstruction, chain, and ownership behavior.

After survivor placement, finish the entire explosion chain against its immutable occupancy
snapshot. Then test living actors' final footprints against the resulting flames. Newly created
fire does not retroactively burn the path traveled before that fire existed. Decide the round
only after all damage phases, so a temporary sole survivor cannot win before another same-tick
explosion kills them.

One living actor wins; zero means a draw. If both remain alive after exactly 10,800 Running
updates, the round is a timeout draw. Elimination on that last update takes precedence over
timeout. Boundary contact without positive-area overlap is safe.

## Finality and clocks

A finalized result freezes actors, world layers, running-update count, and outcome. Later
steps still advance the absolute tick, allowing a future room clock to carry result displays
and lifecycle deadlines. Gameplay commands have no effect after finalization. Existing input
batch and tick-overflow errors remain atomic and are checked before any mutation.

This is a domain result, not a network disconnect/forfeit policy. M3 will report interrupted
preview sessions without inventing wins; M5 adds the first-to-three match lifecycle.

## Verification

Cover swept and stationary fire contact, turn corrections, safe footprint edges, pre-placement
death, self-elimination, simultaneous death, dead-owner bombs, full-chain result ordering, exact
timeout boundaries, and frozen results with later scheduled bombs. Keep prior fuse, occupancy,
and movement regressions passing, adapting fixtures at the appropriate layer rather than
weakening damage ordering to preserve an old test.
