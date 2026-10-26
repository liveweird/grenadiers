# ADR 0026: Server-owned solo controllers and bounded planning

Status: accepted.

## Context

A CPU must obey ordinary game rules without acquiring a human credential, blocking room ticks,
reading concealed information or making old decisions authoritative after a lifecycle change.

## Decision

Gameplay v12 exposes immutable room mode and controller metadata. Solo fixes human One and CPU
Two. Keep human network bindings separate from the CPU controller, reject public solo joins,
and base retention on human eligibility. Preserve existing readiness, reconnect and finality
rules; settle human departure before canceling CPU work and destroying the room.

Project neutral observations from checked recipient state. A process-wide bounded worker pool
plans ordinary intents using independent seeded randomness and fixed work. Room owners never
wait for a worker. Fence results by room, physical world, command epoch, request and age; discard
stale work. Commit memory and randomness only with accepted current input.

After accepting a bomb, execute its already planned bounded escape route without waiting for
another worker. This finite waypoint controller performs no goal/path search or random draws;
movement and scheduled Stop still use ordinary runtime input. Worker contention therefore cannot
invalidate the route's own scheduling allowance. New hazards can still invalidate its game-world
assumptions. General timed routing and tactical search remain separate policy work.

Solo replay format 3 adds private policy/seed provenance to the header; accepted inputs remain
the replay source. Retain formats 1/2 and production's disabled journal allocation.

## Consequences and verification

Deploy server and browser together. No database migration, fake CPU capability or new client
physics authority is introduced. Test controller isolation, bounded work/cancellation, stale
results, delayed-worker escape, settings/start/rematch/reconnect/departure, accepted-input replay,
recipient privacy and the paired browser flow. Keep existing exhaustive tests and coverage floors.
