# ADR 0005: Immutable explosion occupancy and timed flames

Status: accepted for the second M2 increment.

## Decision

Resolve each tick in explicit phases: expire flames, accept inputs, move actors, place bombs,
plan all detonations, then apply the complete plan. Damage and round results follow in the
next increment; they will consume movement traces and the resolved flame layer.

The explosion planner borrows the complete post-placement world immutably. Seed it with bombs
whose deadlines are due and bombs on active flames. Queue each bomb at most once. Each bomb
creates its center flame and four cardinal rays up to its captured range. A hard wall or arena
edge stops a ray without a flame; a soft block receives a flame and stops it; another bomb
receives a flame, is queued, and stops it. Keep every bomb and block in the occupancy snapshot
until the entire chain is planned, including already-visited bombs and marked blocks.

Apply the sorted plan once: remove detonated bombs, clear their actor exemptions, remove hit
soft blocks, and merge flame cells. Capacity comes from remaining bomb ownership, so detonation
releases it exactly once. Because placement happens first, a slot freed by detonation becomes
available on the following tick.

Store flames as cells with exclusive expiry ticks. A flame created at tick `t` is active through
`t + 29` and expires before movement at `t + 30`. Overlap keeps the later expiry. Old flames do
not grow new rays; they can only trigger a bomb subsequently present on their cell. New bombs
placed on active fire participate in that same tick's plan.

Expiry uses saturating addition at the final representable `u64` tick. This creates a
deterministic terminal state without an error after mutation; no later successful step can
observe an extended lifetime because advancing past that tick already fails atomically.

## Verification

Use exact fuse/expiry boundary tests, obstruction fixtures, multi-bomb chains, simultaneous
block hits, overlapping flames, and capacity-release checks. Permutation tests change bomb IDs
assigned to fixed geometry, since insertion order alone cannot challenge a sorted map traversal.
Keep transport, rendering, damage, and scoring out of this increment.
