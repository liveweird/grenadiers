# ADR 0028: Solid players and one-mob respawn lairs

Status: accepted.

## Context

Periodic multi-mob spawning and pass-through player movement make arena combat crowded and
reduce meaningful positioning. Existing rules8/gameplay16 encode those semantics and synchronized
lair timers, so changing them in place would invalidate old contracts and recorded inputs.

## Decision

Use rules9/gameplay17/CPU21 and test-only replay8. Keep old profiles closed. Each lair owns one
living mob or an independent spawn deadline. First spawn and post-death replacement use the
configured delay; successful birth suspends its timer. Obstructed eligible births retry next
tick without accumulating spawns or rotating direction priority until success.

Living players reserve distinct center cells and nonoverlapping footprints. Existing round
priority resolves voluntary movement against every living start and accepted piecewise sweep.
Forced Bracelet movement keeps those reservations; physical face contact supports shoving
without overlap. Teleporter swaps validate the proposed pair atomically against third players.

CPU21 plans against recipient-visible player blockers and seeks reachable firing positions
against all visible rivals. It retains checked bomb escape, bounded work, mob-contact proofs
and existing origin recovery. Unexpected rival movement can invalidate execution; stationary
occupancy assumptions do not promise survival against arbitrary future opponent actions.

## Consequences

The new wire has a fixed cap of one and nullable per-lair timers; clients must refresh after a
paired deployment. Hidden actors remain physical blockers without revealing positions.
Generators, layout, lifecycle and randomness identities remain unchanged. Replay8 consumes
accepted inputs without rerunning the CPU; no durable-data migration or production recorder is
introduced. Focused collision, lifecycle, privacy, combat and rollback regressions accompany
native server/browser acceptance.
