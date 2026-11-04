# Reactive Hard CPU evidence — 2026-11-04

The [backlog](../../BACKLOG.md#single-player) owns work status. The behavior and accepted risk
are described in [CPU contracts](../game/cpu-contracts.md#reactive-hard-planning) and
[ADR 0030](../decisions/0030-reactive-hard-planning.md).

## Conservative baseline

The investigation started from published v1.3.23, source `e336c703`. Its live family uses
Easy/Standard62 and Hard64 with generator7, pit-layout2, gameplay21 and ContactV2.

The captured foreign-bomb basis at tick 234 has a useful plant, but the explicit East-refuge
candidate returns `NoRoute` after 3,232 work units. Its forecast retains motion uncertainty;
this is not simply an exhausted search, nor proof that every possible route fails. The
unrestricted candidate search separately exhausts the 65,536-unit allowance.

The current generated 18-case reference matrix queued and delivered 3,395 jobs, failed none,
and physically placed 113 bombs. It contained 381 Farm/Approach observations with a foreign
bomb and spare capacity, 194 already at the approach. Every candidate failed the visible-bomb
support guard. The Hard-with-lairs cases supplied no such Farm candidate; the Hard/no-lairs
tick 585 example was already physical-origin recovery and cannot count as new planting evidence.

These findings motivated the explicit Hard-only measured-risk decision. Easy/Standard and
archived policies retain their semantics; Hard65 uses a separate runtime family and replay34.

## Interrupted escape exposed by the controlled match

The controlled comparison uses the same generated seed 73 arena, concealed contents and human
input schedule. Other CPU controllers remain active. No concealed loot or private controller
history is edited to manufacture a plant.

An intermediate Hard65 implementation placed bomb 6 at `(8,1)` on tick 292, detonating on tick 472
with flames expiring on tick 502. It contributed to destruction of the named block `(6,1)`.
Its certified route stayed near the top row. A rival placed bomb 7 at `(6,7)` on tick 439; its first
publication on tick 441 correctly revoked the old route under the global new-bomb rule.
The CPU stopped near `(11,1)`, could not obtain a conservative replacement escape, and died
in its own blast on tick 472. The complete outcome ledger recorded one reactive placement and no
survivor. Selecting another seat or only a surviving certificate would conceal that failure.

This justified adding fresh, nonmanual OwnEscape proofs. It did not justify ignoring the new
bomb. The replacement proof must use current two-sample histories, include actual owned bombs,
allow observed finite flames while checking their exact hazards, and cover the owned cohort's
flame-expiry update. Further new bombs still revoke it. A canceled program grants no input
authority while a replacement is being planned.

## Review corrections

Independent review and integration checks corrected publication-versus-physics scheduling,
binding to the exact checked publication, stale stationary history, route/contact handoff,
nonplant strategic-admission filtering, lair-aware demolition distance, ordinary grace expiry,
and shared renewal accounting. Reactive actor validation is charged once and executed once.
The primary Attack commitment remains intact while a separate sealed block purpose describes
forward demolition.


## Accepted physical trace and unsafe-position regression

The final controlled trace retains bomb 6 at `(8,1)` on tick 292 and adds a second useful reactive
placement on tick 550; their finite flames clear at ticks 502 and 760 respectively. Both placements survive
and make subsequent progress. The new-bomb publication on tick 441 still cancels the first route.

The final trace is already safely stopped at `(11631,1536)` on tick 441 and 444. A genuinely admitted
ReactiveRoute uses stationary waiting through tick 502 and completes on tick 503. No extra manual action
is introduced. This is not evidence that movement saved the identical earlier unsafe pose:
the earlier failing trace's lethal position was `(11478,1536)`.

A separately labelled counterfactual changes only the recipient position in the captured
public basis at tick 444/history to that unsafe position. Hard65 produces an East escape using 12,140
work units, seven start lanes `[445, 452)`, and survival deadline 503. The later foreign bomb
expiry at tick 619 does not extend the owned-cohort objective. Plain admission rejects this certificate;
explicit reactive admission accepts it and its scheduled movement commits. The certificate
carries no manual action, plant payload, placement tick or new origin grace.

The same captured evidence checks new/moved/paused bombs, pickups, Kick and Teleporter changes,
plus wrong-profile copied certificates. These negative API tests complement the actual match;
the counterfactual is not an authoritative match replay.

## Fixed generated comparison

The reference uses all items, multiplier 5, generator7/pit-layout2, 840 updates, three CPU seats,
all three board sizes and lairs both ways. Easy/Standard62 controls use seeds 71/72; Hard64/65
pairs use seed 73. All jobs remain bounded by 65,536 work and report no worker failure.

Each arrow is Hard64 → Hard65. Eliminated seats counts CPU seats observed dead at least once,
not elimination events. Switch counters describe reported strategic reasons, not time stationary.

|  Board  |  Lairs  |  Plants  |  Blocks destroyed  |  Ever-eliminated seats  |  Stalled switches  |  Completed-objective switches  |
|  ---  |  ---  |  ---  |  ---  |  ---  |  ---  |  ---  |
| 11×9 | Off | 3→3 | 3→3 | 0→0 | 0→0 | 3→3 |
| 11×9 | On | 4→4 | 4→4 | 2→3 | 4→3 | 0→2 |
| 13×11 | Off | 6→6 | 9→9 | 3→3 | 0→0 | 3→3 |
| 13×11 | On | 5→4 | 5→4 | 3→1 | 2→4 | 0→3 |
| 15×13 | Off | 6→6 | 9→9 | 2→2 | 0→1 | 3→3 |
| 15×13 | On | 3→3 | 3→3 | 1→1 | 0→0 | 3→3 |
| Total | — | 27→26 | 33→32 | 11→10 | 6→8 | 12→17 |

First planting remains217/190/217/190/217/190 respectively. Final full-cohort survival remains
true only for the 11×9 and 13×11 no-lair cases. Maximum work remains within 65,536, and the
instrumented admission-rejection counters report zero. This small comparison is mixed: one
lair case improves substantially, another loses an additional seat, and aggregate planting is
slightly lower. It does not establish a general aggression or difficulty improvement. The
controlled trace demonstrates the new conditional capability; broader balance needs playtesting.


## Focused checks before the full release gate

The queued controlled match, counterfactual route, copied-profile/mutation checks, fixed
comparison, worker parity, owner-family admission and ordinary carried-intent route regressions
pass. Runtime-family retention passes both tests; private replay34 passes five tests, including
CPU input fences, partial publication and shutdown departures. Independent review findings are
resolved.

The ignored native Hard65 production-pacing test was run separately with no competing test or
build workload. Both lair settings pass in 31.74 seconds, with three CPUs moving/planting,
matching placement counters, all initial bomb owners surviving their complete flame windows,
and maximum work at 65,536. This is an operational pacing check, not a claim of late-round
survival or playing strength.

The [published release notes](https://github.com/liveweird/grenadiers/releases/tag/v1.3.24)
identify the final tested source, complete checks and deployed image.
