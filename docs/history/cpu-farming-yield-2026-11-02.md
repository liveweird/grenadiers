# CPU farming-position comparison — 2026-11-02

This controlled characterization preserves Hard58 behavior before a same-target yield refinement.
It is not a restored match or a claim about hidden loot. Status belongs in the
[backlog](../../BACKLOG.md#single-player).

## Legal controlled scene

CPU Two naturally collects BlastRange at `(5,5)`, raising range to three. Soft blocks `(3,7)`
and `(7,7)` are the relevant farm targets. Human One is boxed at interior cell `(11,9)` by four
other soft blocks, preventing a reachable attack firing origin. Canonical opposite spawn metadata
and its clear reservations remain intact. An initial proposed corner box was rejected by the real
Arena constructor; no constructor invariant was relaxed.

All items are enabled; storm, lairs and pits are disabled. Seed 58 and empty concealed/pending loot
maps are explicit fixture assumptions. The observation does not disclose those private maps.

## Core baseline and physical proofs

At tick 1, Hard58 chooses Farm block `(3,7)`, approach `(3,6)`, initial West, no manual action or
execution, for 21,360 work. Real simulation walks reach both `(3,6)` and `(4,7)` in 60 movement
updates, followed by a confirmed Stop at 62. Both approaches have graph distance 3.

Separate raw complete plant queries, including target usefulness, cost 3,002 and 21,854 work
respectively. Both place at 69, detonate at 249 and cover survival through 279. Admission uses a fresh
actual observation at 68; committed commands then run through the real State. The first origin
destroys only `(3,7)`; the other destroys both target blocks. Exact total destruction excludes box
collateral. Bombs/flames clear and the CPU remains alive.

Raw affordability is not live-planner affordability. The full Hard58 wrapper plants at `(3,6)`
for 3,764 work, but refuses planting from nominal `(4,7)` for 39,979 work. Its bounded first candidate
receives 19,435 units, below the raw Stop query’s 21,854; later movement remains checked.

The real South-path intermediate `(5,7)` is the useful opportunity: after a real 40-update walk
and Stop at 42, the existing full wrapper selects South+Bomb for 36,436 work. It plants at `(5,7)`
at 49, detonates at 229, and its physically executed/admitted route survives through 259 while both
relevant walls are destroyed. No extra refuge machinery or larger work limit is justified by
this witness. Future ranking must reproduce the early plant through the actual queued owner.

## Queued owner before-state

The test-only duel wrapper preserves normal admission/countdown/publication and all runtime
scene-install checks. It captures a real ranged planning job at tick 183/settled Stop 183 and
asserts no escape origin before shadowing ordinary planning plus the existing Teleporter selector.
Actual dispatched intent and work match that same indexed job.

Hard58 selects Farm `(3,7)`, nominal approach `(3,6)`, first West, at 21,485 work. It naturally
plants earlier, at `(3,5)`/232, detonates at 412 and resolves flames at 442. The target disappears, the
second block remains, exactly one block is destroyed, and an open blast cell `(3,6)` is observed
with the exact flame expiry before hazards clear. The CPU survives; all 4 jobs are delivered,
with zero failures/cancellations/drops or admission rejection counters.

The shared probe and duel builder retain the existing four-seat exit-pressure and Teleporter
fixtures. Core and server independent reviews are clear after strengthening actual-flame,
no-origin and exact job-count assertions. Debug/release each pass five Core cases and seven
relevant owner regressions; formatting, strict workspace Clippy and repository checks pass.
Logs use `target/quality-review/farm-yield-diagnostic-*` in the main checkout.

## Hard59 refinement and queued owner comparison

Hard59 compares at most four legal adjacent approaches only after selecting the existing Farm
block target. Equal graph distance is required; a strictly greater visible block yield wins.
Exact Standard, Piercing and Power geometry includes current placement modifiers. Unsupported
scenes, partial comparisons, ties, failed origins and farther approaches retain baseline guidance.
The comparison uses the unused original pressure allowance and its existing geometry; it adds no
second search pool, forecast or RNG draw. Retained approaches keep the existing progress deadline.

The initial Core choice changes from `(3,6)`/West to `(4,7)`/South for 21,370 work, ten more than
Hard58. The useful intermediate `(5,7)` still uses the existing 36,436-work complete plant.

Through the real queued owner, the same basis/settled Stop 183 selects `(4,7)`/South for 21,495
work, versus the archived 21,485. The CPU naturally plants at `(5,7)`/232, detonates at 412 and
survives through flame end 442. Both `(3,7)` and `(7,7)` disappear, exactly two blocks are destroyed,
actual flames are observed and clear, and all four jobs are delivered with zero admission errors.
The paired Hard58 test retains its West route, early `(3,5)` plant and one-wall yield exactly.

Independent review identified a replaced-approach progress comparison; Hard59 now measures the
validated retained approach, while archives keep their old semantics. A rival-occupancy plus
subcell-movement regression prevents false deadline refresh. A 5,200-work partial-comparison
case charges the attempted work and returns the exact archived baseline guidance. The owner
comparison pins the exact basis, work, plant/flame ticks and job counts. Both reviews are clear.

Release acceptance will be recorded in the [release record](../releases.md#v1318--2026-11-02).
