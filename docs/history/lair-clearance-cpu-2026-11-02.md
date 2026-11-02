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

The unchanged Hard63 broader acceptance is red. Room 4028, seed 73, 13×11, all items and lairs,
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

## Hard64 queued-owner comparison

Prototype commit `afd6a6460f9efeb5182debbc1da1a5a7a8aef31f` adds a versioned, initial-choice
demolition opportunity. Incoming strategic memory must be empty and ordinary planning must
have just selected Attack/Approach. It reuses the preferred candidate's complete certificate:
a useful attack stays an attack; an otherwise useless attack may adopt a visible block hit by
that same blast. Existing commitments retain their original planner. Failure continues through
the four remaining candidates using their original fixed reserves, without repeating the
prefilter or preferred proof. No hidden loot, RNG draw, work increase or safety-guard relaxation
is introduced.

The same room-4028 queued owner now selects Farm block `(11,3)` at tick 183, with a complete
Plant certificate ending at 400. The real bomb is placed at `(11,1)` at 190 and detonates at
370. The target wall remains through 369, actual flames appear during 370–399, and at 400
the target is absent, flames are empty and CPU Two is alive. Work remains within 65,536.
The explicit Hard63 before-test retains `[0,2,2]`; the current Hard64 prototype matrix passes
its unchanged operational assertions, including the separate 15×13 case.

Independent review found no actionable defect after correcting the initial exclusive-demolition
branch and completing profile capabilities/dispatch. A nonblocking coverage gap remains for
small direct tests of exact reserve expenditure and useful-Attack output parity. This is an
unmerged generator-5 comparison; it does not establish acceptance for weighted generator 7.

## Checks recorded at the diagnostic checkpoint

Formatting, strict workspace Clippy and repository checks pass. Core tests record 1,981 passes,
two ignored tests and two passing doctests. Focused server own-blast, complete recovery-horizon,
zero-bomb diagnostic and prototype replay cases pass. The later demolition counterfactual passes
its focused test with formatting, strict Clippy and source-limit checks.

Local logs use `target/quality-review/layout-checkpoint-*`,
`layout-current-v63-matrix.log`, `layout-zero-bomber-placement-classification.log` and
`layout-zero-bomber-demolition-counterfactual-final.log`. These are checkpoint results, not a
full integrated server/native/replay-mask release acceptance.

The Hard64 focused checkpoint additionally passes 26 Core contact-punch cases, its real
demolition witness, the unchanged Hard63 witness and three prototype replay cases, plus strict
workspace Clippy, formatting and source limits. Its log is
`target/quality-review/layout-v64-focused.log`.

The first broader debug run exposed three archived fixtures inheriting the new 60/300 spike
defaults and a stale current-profile expectation. Test-only correction `3c93be0f` reuses the
published `ActiveWithFormerSpikeTiming` fixture for the archived 36/180 trajectories while
retaining current defaults elsewhere. The admission case now accepts Hard64 and rejects
Hard63/Hard59. Exact work, movement and pit assertions are retained. The five focused cases,
formatting, strict workspace Clippy and source limits pass; independent review is clear.

Frozen commit `3c93be0f95471093586bdb4f935cb7f8abc3eba0` subsequently passes ordinary workspace
tests in both debug and release: 1,981 Core cases, 790 server cases and two doctests per profile.
Two Core and 15 server cases are ignored; the exhaustive item-mask case and four isolated
timing-sensitive cases are explicitly excluded. The worktree is clean before and after.
Logs are `layout-v64-checkpoint-debug.log` and `layout-v64-checkpoint-release.log`; the original
failure is retained as `layout-v64-checkpoint-debug-before-fixture-fix.log` under
`target/quality-review/`. No coverage/native/mask release acceptance is claimed for this checkpoint.
