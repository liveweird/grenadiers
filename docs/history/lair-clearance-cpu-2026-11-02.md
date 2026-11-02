# Lair clearance and CPU investigation — 2026-11-02

This record describes a controlled, **unreleased prototype**, separate from the live v1.3.22
game. Open work and priority belong only in the [backlog](../../BACKLOG.md#arena-layout-and-environment).

## Source and compatibility boundary

The `feat/lair-clearance` branch preserves three focused changes: connected lair clearance
(`2cc15272`), certified CPU recovery (`4d4d1750`) and explicit server construction/replay
profiles (`c3fde037`). Diagnostic commit `3bbc4f16` adds the demolition counterfactual below.
These commits use the earlier gameplay-v20, unweighted-drop base for controlled comparisons;
they are not release candidates for the current protocol.

The prototype's generator 5 adds an RNG-free postprocess to the existing reflected layouts.
It clears each active lair's surrounding ring and connects that space to the player-opening
network, preferring fewer removed blocks and then shorter paths. Only loot belonging to
removed blocks is discarded. Pit layout 2 excludes the rings and preserves usable exits.
Legacy constructors retain their original layouts and random draws.

The prototype's private replay 32 was allocated before the separate weighted-drop feature
published that number. It cannot be integrated under that identity. The shipped generator 6,
gameplay v21 and weighted replay 32 retain their published meaning.

## Own-bomb recovery witness

In the controlled seed-73, 13×11 lair episode, archived Hard61 accepts a plant, then loses its
execution after another bomb changes the scene. A recovery movement crosses its intermediate
axis coordinate without landing exactly on it; the old continuation stalls before its own
bomb detonates at tick 511.

The versioned Hard63 prototype accepts reaching or crossing that intermediate coordinate
through unchanged movement physics and complete route certification. It retains a charged
stationary safety suffix rather than returning prematurely to ordinary planning. Actual
execution maintains authority from ticks 399 through 541, survives detonation and the full
flame window, and completes at 542. Recovery uses 19,486 of 65,536 work units; peak renewal
uses 1,434 of 8,192. Archived behavior remains available for the before comparison.

Independent review found a final-checkpoint boundary error. The corrected suffix permits
only its sealed last update after all common scene/pose checks. Exact-budget, one-short,
bomb/effect/Teleporter mutation and physical final-update regressions pass in debug and release.

## Zero-bomb actor and useful demolition counterfactual

The unchanged broader acceptance remains red. Room 4028, seed 73, 13×11, all items and lairs,
Hard63, world digest `7778025744984032398`, records actual bomb placements `[0, 2, 2]`.
The first CPU produces no bomb; the test does not substitute surviving-bomb counts for actual
placement events or weaken productivity assertions.

Classification of 30 actual planning jobs rules out the proposed candidate-budget explanation:
every supported placement is geometrically useless against its selected opponent. The only
useful attack geometry, at tick 636, is rejected by the existing visible-bomb Teleporter support
guard. This evidence does not justify raising budgets or relaxing that guard.

At tick 183, however, the preferred West candidate can hit visible soft block `(11,3)` from
`(11,1)`. A diagnostic rerun with the existing Farm usefulness rule certifies the same placement
within its original 32,371-unit candidate slice: proof cost 12,815, total job cost 17,238 of
65,536. Expected placement/detonation/full-clearance ticks are 190/370/400.
This is a **counterfactual certificate**, not evidence of a live goal switch, queued admission,
actual placement or authoritative survival.

## Checks recorded at the diagnostic checkpoint

Formatting, strict workspace Clippy and repository checks pass. Core tests record 1,981 passes,
two ignored tests and two passing doctests. Focused server own-blast, complete recovery-horizon,
zero-bomb diagnostic and prototype replay cases pass. The later demolition counterfactual passes
its focused test with formatting, strict Clippy and source-limit checks.

Local logs use `target/quality-review/layout-checkpoint-*`,
`layout-current-v63-matrix.log`, `layout-zero-bomber-placement-classification.log` and
`layout-zero-bomber-demolition-counterfactual-final.log`. These are checkpoint results, not a
full integrated server/native/replay-mask release acceptance.
