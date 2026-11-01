# Late CPU farming cycles — 2026-11-01

This is investigation evidence, not a certificate of a missed safe action. Work status remains in
[the backlog](../../BACKLOG.md#single-player).

## Fresh CPU45 match

The exact v1.3.9 image was observed through public snapshots and CPU diagnostics in an all-items
match with one human and three Hard CPUs. The match reached a result in 179 seconds without
JavaScript errors. The retained local capture is
`target/quality-review/continuation-v139-long-match.json`.

An exposed Teleporter at public tile `(3,9)` (zero-based) first appears in saved sample 28/tick 1861 and remains
through the last sample. No bomb is planted after tick 1628. Late scenes have no active bombs;
all three CPUs hold Punch, have capacity one/live zero and no effects. The board retains 56 blocks.
Every late diagnostic reports Farm / Approach / Stalled.

Same-cell summaries do not establish continuous stillness. At ticks 6001, 6061 and 6121:

| Seat | Exact public poses |
| --- | --- |
| Two | `(9843,1536)` west → `(9728,1421)` north → `(9843,1536)` east |
| Three | `(5632,11968)` south → `(5210,11776)` west → `(5824,11776)` east |
| Four | `(11891,10752)` east → `(11021,11776)` west → `(11776,11481)` north |

Seats two/three repeat subcell patterns while selecting adjacent farming targets. Seat four
circulates among a small set of cells. An exposed Teleporter globally rejects current planting
in `cpu/survival/plant_tactic.rs`; `joint_refuge.rs` also excludes it from farm-origin deferral,
and `lair_contact_topology.rs` excludes it from topology certificates. Pickup guidance in
`cpu/policy/goals.rs` gives Teleporter no collection value and forbids deliberate transit across it.
This confirms a support boundary behind the passivity, not a completed alternative safe route.

The fixture and focused tests in `core/src/cpu/lair_contact_v139_teleporter_*` preserve sample 99.
Default/all-items admission settings and settled-Stop/execution ticks remain explicit assumptions.
The exposed Teleporter rejects the plant contact proof after eight charged scans; removing only
that pickup permits the same contact certificate. This isolates the support guard, without
claiming actual collection or a physical trajectory from the captured match.

Do not remove those guards without a replacement proof. Intentional collection would need to
cover every possible live partner, both endpoints, retained motion and post-teleport mob/hazard
contacts without observing private partner-selection randomness. Destruction would need to
cover collection throughout the certified bomb escape, not merely show that the eventual blast
reaches the pickup.

## Fresh CPU47 match

The exact v1.3.10 candidate image was observed through public snapshots and CPU diagnostics with
one human and three Hard CPUs. The round reached a result in 180 seconds with no JavaScript
errors. `target/quality-review/teleporter-v1310-long-match.json` retains 181 public samples and
319 diagnostic responses. The CPUs planted 3/10/11 observed bombs by seats Two/Three/Four.
This is not a matched balance comparison.

Two exposed Teleporters disappear with actual public swap markers followed by renewed bombing:
`(11,2)` appears at sample 16/tick 1141; Two/Three share marker 1931 and Three plants at 1943.
`(11,10)` appears at sample 54/tick 3421; One/Four share marker 4025, followed by bombs at 4037
(Three) and 4298 (Four). The public capture does not identify which internal certificate path ran.

A third Teleporter at `(3,2)` first appears at sample 88/tick 5461 and remains through 10981;
there are no later bombs or new teleport markers. Preserve sample 87/tick 5401 and diagnostic
responses at 5413/5449/5488 as context. Four is already at the unchanged exact pose `(1536,2724)`,
south, from sample 86/tick 5341 onward. Its diagnostics become unavailable at 5449; this does not
identify planner refusal, work exhaustion or a stopped controller. Two continues moving between
saved samples while alternating Farm/Approach/Stalled between blocks `(3,8)` and `(4,9)`, both
from approach `(3,9)`. Three dies between saved ticks 4501 and 4561 while holding Wall Pass; its
4510 diagnostic remains stale. The capture does not establish the death cause.

Sample 88 has 48 blocks, exactly one Teleporter, visible empty bombs, no flames or Oil, and the
complete visible roster (One/Two/Four alive, Three dead). Four has speed zero, capacity one/live
zero/range two, no equipment, weapon or effects, and is unfrozen. Two unfrozen mobs are at
`(5632,6656)` and `(9728,6656)`; both lairs have one living mob and no spawn countdown. Four pits
are idle; the storm is at `(3648,7680)`, east, leg remainder 120. A static floor search gives Four
`(1,2)→(1,1)→(2,1)→(3,1)→(3,2)`; Two has no floor-only path from its separate eight-cell component.
This is reachability, not a certified physical approach.

The required follow-up was to record real recipient observations, admitted Stop and later physical Stop
confirmation, controller memory, worker work/delivery/cancellation, admission outcomes and a
continuous productive trajectory. Do not reconstruct hidden residuals, RNG history or controller
state from public JSON, assign a saved tick as the Stop clock, or relocate the actor to a favorable
endpoint. A declared controlled scene may characterize the geometry without claiming restored
match provenance. At this capture-only stage, no missed safe action or rejection-memory defect was established.

### Conditional CPU47 scene characterization

`core/src/cpu/lair_contact_v1310_late_teleporter_{fixture,tests}.rs` preserves every public
observation field from sample 88/tick 5461. The fixture-local world identity and default/all-items
admission configuration are explicit assumptions; private actor facts remain unknown. The tests
also explicitly assume fresh utility/hazard memory, a fixed test RNG seed and settled Stop at
5461. None of those private execution facts is recovered from the capture.

Under those premises, the exact no-origin Hard47 worker sequence spends 3,804 work on its ordinary
fallback and selects a typed Teleporter step at 14,176 total within the shared 65,536 allowance.
Guidance targets Collect `(3,2)` and moves North; a direct certificate is Approach with the assumed
Stop provenance. The direct selector succeeds with exactly 10,372 work; one less preserves the
unchanged ordinary fallback. This establishes conditional selection, not actual captured
admission, a complete physical approach, a missed safe action or the cause of unavailable intent.

Three focused regressions, strict Core test Clippy, formatting and repository checks pass.
Independent review matched every fixture field to the raw sample. Production behavior and the
accepted v1.3.10 image remained unchanged at that checkpoint. The following owner reproduction
supplies the missing execution evidence.

## Continuous owner reproduction and CPU48/49 repair

The controlled production-owner episode in
`server/src/game/rooms/roster_owner/late_teleporter_owner_{support,tests}.rs` keeps sample 88’s
CPU pose, 48 soft blocks, Teleporter, dead seat Three and fixed lair/pit geometry. Initial clock,
controller/RNG history, the departure used to create the dead actor, and subsequent cloud/mob
states are declared controlled assumptions. This reproduces the refusal mechanism, without
claiming restoration of the captured private match.

Archived Hard47 delivers valid typed recovery plans but fences the first admission at tick 187:
compatibility requires every roster member to have been alive, although certification permits
an unchanged dead member. No movement, collection or intent memory commits. A second mismatch
requires the confirmation tick to equal the physical Stop tick, although the controller records
the later publication that actually confirms it.

CPU48/49 isolate both corrections. An already-dead visible actor must retain identical public
facts. The sealed Stop still executes at admission `A+2`; a later confirmation is accepted only
between that physical deadline and the current observation, before the original safety expiry.
Every scene/contact, command, work and authority check remains. CPU46/47 retain their old behavior.

The new Hard49 episode completes 74 admitted Move/Stop pairs with no invalidation fences,
collects at tick 626, plants at 635, destroys a block and remains alive through tick 1381.
All 84 jobs are delivered without failures/cancellations and within 65,536 work. The witness
checks actual next-update Stop poses, later confirmation and committed collection intent.
Focused Core tests additionally reject resurrection, changed dead facts, premature/future
confirmation, omitted Stop, expired authority and changed scenes; archived admission remains
closed. Private replay24 binds CPU48/49 while replay23 retains CPU46/47.

## Earlier CPU43 match

In `target/quality-review/births-v138-long-match.json`, sample 90/tick 5580 has seat three at
`(4646,5632)`, range two, capacity two/live zero, Punch and no effects or exposed Teleporter.
Later diagnostics alternate block `(3,4)` from `(3,5)` and block `(5,5)` from `(4,5)`.
Both nominal plants destroy `(5,5)`, opening the separator toward a central mob component.
Selected-target removal alone therefore cannot establish safety.

A farther endpoint, block `(1,5)` from `(1,6)`, preserves that separator in static analysis.
It remains a candidate until the complete approach, contact checks and physical execution are
certified. A possible one-entry rejection-memory cycle must not be called the cause of a missed
safe action without that witness. The two captures need separate reproductions.

### Preserved public-scene regressions

The fixture and three focused tests in `core/src/cpu/lair_contact_v138_farm_witness_*`
preserve the public arena and explicitly assumed all-items settings. The capture does not retain
the private settled-Stop clock or motion residual; every certificate also assumes that stopped
basis and execution tick. These are conditional tests, not restored-match execution.

Under that premise, a productive route at the original public pose fails both mob contact proofs.
One westward 12-update prefix passes route/contact checks. Relocating only the actor to `(1,6)`
allows a contact-safe endpoint plant against `(1,5)`, but the complete approach is not demonstrated.
No alternating-memory guidance defect is established. Focused tests, strict all-target Clippy,
formatting and repository checks pass. Independent review corrected the stopped-clock claim.

## Fresh CPU49 full round

`target/quality-review/roster-v1311-long-match.json` records one 180-second all-items
round with one human and three Hard CPUs, from tick 196 to result 10996. It retains
181 public samples and 318 diagnostic responses; there are no JavaScript errors.
CPU bomb counts are 0/3/3 by seats Two/Three/Four. This is an uncontrolled fresh round,
not a matched comparison with previous releases.

Two exposed Teleporters at `(1,7)` and `(13,7)` persist from sample 20/tick 1381 through
the end; the last observed CPU bomb was placed at 674. Both guidance and certification
require exactly one Teleporter, while ordinary planting rejects any exposed Teleporter.
This identifies another explicit support limit, not an established safe unexecuted collection.

Seat Two also alternates farming targets `(11,2)` from `(11,1)` and `(9,1)` from `(10,1)`
before the pickups appear. It never plants, traverses four top-row cells and makes 240
observed cell changes; describing it as continuously stationary would be incorrect.
Seat Three has a long same-cell interval before moving again; a cell summary alone does
not establish unchanged exact pose. The public capture does not contain private controller,
queue, Stop or refusal history. Preserve separate original-pose owner witnesses before
claiming a missed safe action or changing farming memory.

## Multi-Teleporter extension characterization

The opt-in v50 certificate preserves fresh CPU49 sample 20/tick 1381 in
`core/src/cpu/routing/teleporter_step_v50_fixture.rs`: 40 blocks, two Teleporters, four living
actors, cloud, lairs, mobs and pits. World identity and stopped-controller history are explicitly
assumed. Seat Three’s actual guidance selects South toward `(9,10)` and target `(13,7)`;
the certificate proves nominal progress. East can certify a blocked/no-move outcome and is not
used as evidence of productive guidance.

The full Hard51-equivalent sequence spends 12,871 work on ordinary planning, then 26,168 total
on selection and a valid Approach; selector work does not alter RNG. Exact Approach proof work is
12,903; a safe counterfactual Collect costs 15,156. Exact budgets succeed and one-short budgets
fail. The actual Collect selector’s first frozen slot is 16,212, sufficient for that proof.
Unselected-pickup reach at the full bound 1,537 rejects; distance 1,538 permits the same Approach.
These are conditional certificate tests, not restoration of private captured history.

The separate controlled two-player owner episode shares the existing driver builder, adds a
second distant Teleporter and uses actual worker/queue/Stop execution. All three difficulties
collect at ticks 247 and 1513 with 222 admitted moves, no fences or failed work. Hard plants at
1753 and wins while alive. Standard/Easy plant at 1576, destroy a wall and survive flame expiry.
The witness enforces a single physical round, distinct collection ticks and immediate survival
of the unselected pickup. An early assertion waited across round-result freeze and inspected a
new-round opponent; correcting that test did not change the production engine.

CPU50/51 isolate the extension; CPU48/49 retain exact-one-target planning. Private replay25
reuses replay24’s accepted-input recording/validation implementation with independent closed
50/51 identity constants. This supports multiple distant pickups without guaranteeing completion
of every multi-Teleporter scene or resolving the separate early farming cycle.

## Fresh CPU51 concurrent-load round

`target/quality-review/multi-v1312-long-match.json` records an uncontrolled fresh round from
running tick 222 to result 1576 (22 seconds), with 24 saved public samples, 29 diagnostics,
17 observed CPU bombs (5/6/6) and no browser errors. This ran alongside release compilation,
mask checks and other browser smoke tests; it is not an isolated latency or balance comparison.

At saved tick 1560, three range-two timed bombs placed at 1396 have 16 ticks remaining:
`(13,3)` owned by Two, `(3,9)` by Three and `(11,9)` by Four. The saved footprints overlap their
predicted blast rays at `(13,1)`, `(3,10)/(3,11)` and `(13,9)` respectively. The terminal state,
accepted input queues, private Stop/origin and planner/refusal history were not retained, so
this does not establish individual death attribution or queue starvation.

Read-only source tracing finds no simple range or clock mismatch: simulation and forecast share
`visit_blast`, observation clocks are physical ticks, and both retain flames until detonation
plus 30. An intact planted route spans through 1606. A concrete mechanism to discriminate is
simultaneous foreign plants invalidating an exact expected-bomb scene, followed by recovery
whose origin can expire after leaving its bomb footprint and whose ordinary pulse safety
window is shorter than the full bomb lifetime. Opening-scene recovery already has a witness;
this later geometry needs a paired actual-owner episode before a defect is claimed or fixed.

## Exact queued CPU51 farming suppression

The controlled owner tests in `early_farm_owner_{support,tests}.rs` preserve the CPU49 sample-0
public arena: 48 blocks, spawn poses, westbound cloud with leg 128, both lairs and the four pit
sites. Fresh private history, simulation seed 47, initial clock and empty hidden loot are declared
assumptions. All eight preselected CPU roots `[0,1,2,3,7,31,47,51]` plant at tick 232 with the same
six delivered jobs and no admission rejection. Public spawn geometry alone does not reproduce
the captured non-planting behavior.

`early_farm_owner_interaction_{support,tests}.rs` compares the same scene with real CPU controllers
for seats Two/Three/Four. Three/Four plant at 205; Two does not plant during 600 updates. Two
reports Stalled at 235 and later selects the other captured target. This demonstrates a
behavioral difference with other CPU activity, without attributing every update to bomb waves
or claiming a continuously restored match. All 263 queued jobs deliver within the work limit;
admission rejection traces are zero for all three. Origin/route/fence observations cover Two.

A test-only immutable callback inspects the actual queued job immediately before execution.
It captures Two at observation/confirmed-Stop tick 234, pose `(11727,1536)`, cell `(11,1)`,
with Teleporter enabled, two visible bombs, 48 soft blocks, no exposed Teleporter, no flames,
running update 54 and no origin. Calling the existing public planner with cloned RNG and the
same observation, memory, hazard history and Stop clock spends 10,443 work and returns North,
no manual action and no execution certificate. Inspection leaves the 263-job trajectory unchanged.

These facts match the existing `ClearArenaCounterPressure` support guard: Teleporter enabled,
visible bombs and any soft blocks suppress planting before route construction, regardless of
distance. This establishes support suppression, not a complete safe missed plant. A Core
follow-up must retain foreign-bomb, contact and unknown future Teleporter obligations; deleting
facts or disabling the item would only create a counterfactual.

The callback, scenario builder and witnesses are test-only. Independent review is clear after
adding the earlier guard facts and limiting causal claims. Three early/interaction tests, two
existing late-owner tests, strict server test Clippy, formatting and repository checks pass.
The integrated Rust/dependency inputs match the tested isolated worktree exactly. Application
behavior and the accepted v1.3.12 image are unchanged; no product version bump or repeated full
matrix/runtime run is needed.

## Quiet pit-entry observations

A separate post-gate capture, `target/quality-review/multi-v1312-quiet-match.json`, retains its
terminal snapshot. It reaches a result after 28 seconds without browser errors. Dead poses for
Three `(3584,7645)`, Two `(11776,5430)` and Four `(11776,7645)` lie on pits `(3,7)`, `(11,5)` and
`(11,7)`. The saved sequence shows warning countdowns before death and raised pits afterward;
this differs from the earlier concurrent-load bomb scene and is not a matched comparison.

Source inspection confirms path-dependent pit clocks are modeled in pulse certification, but
ordinary navigation/goal approaches do not exclude pits. A 12-update pulse is checked through
basis+29, shorter than the default 36-update warning. That permits a locally safe entry without
establishing that staying at the goal is safe through the later rise. A pre-entry queued-owner
witness must establish the accepted sequence before changing policy; these public samples alone
do not identify the last admitted certificate or a missed escape.

## Controlled CPU51 pit-entry failure

The real queued owner fixture `archived_hard51_voluntarily_enters_the_quiet_capture_pit`
preserves the quiet recording's public wall layout, Three's position `(3584,8515)` and pit
`(3,7)`. Other seats are idle humans; empty hidden loot, fresh clocks/controller memory and
seed 47 are explicit assumptions. It is a controlled reproduction, not restoration of the
original private match.

CPU51 selects block `(2,7)` with approach `(3,7)`. Its actual queued basis at tick 198 has physical
Stop confirmed at 198; the checked North pulse has start window `[199,206)`, hold 12 and exclusive
safe-until 227. The real simulation activates the pit at 206, stops the CPU inside it, and kills
it when spikes rise at 242. The preceding tick leaves the CPU alive at the same position.
Delivered jobs stay within 65,536 work and show no cancellation or admission refusal. The pit's
warning delay exceeds the short pulse's proof window: local pulse safety does not establish a
productive strategic destination. A minimal one-wall scene changed pit placement and chose
Attack, so it did not reproduce this failure and is not used as the before witness.

The paired CPU52/53 run preserves that public scene: Hard, Standard and Easy remain alive for
600 updates without touching a pit. The corridor is boxed by hard walls, a soft block and the
pit; the retained plant prover cannot certify the available escape. These runs establish
avoidance, not productive farming or a claim that physical escape is impossible.

A separate productivity fixture removes only soft block `(3,9)`, opening a southward route.
All three difficulty profiles plant at 235, destroy a block and remain alive through flame
expiry at 446. Every pre-plant actor footprint avoids every pit. This distinct fixture confirms
that the restriction still permits useful play when a checked alternative exists; it is not
presented as the identical captured scene. No safety obligation or work allowance is relaxed.

## Controlled concurrent-escape preparation starvation

A fresh CPU53 browser capture contains three range-three plants at tick 944 and three deaths
at detonation 1124. Unlit Oil is visible, but a public capture cannot identify private authority
or reconstruct its original queued inputs.

The controlled seed47 owner episode retains its public 81-block layout and normal spawns.
Three concealed range upgrades and two optional concealed Oil barrels are declared fixture
contents; other concealed contents, fresh clocks and controller histories are assumptions.
Natural inputs and pickups produce synchronized range-three bombs at 862, due at 1042.
Both Oil and no-Oil worlds lose all three CPUs at 1042. Result-state physics freezes there;
the witness does not fabricate later flame expiry.

The actual queue retains origins after route withdrawal at 864 and has confirmed Stop at 867.
Immutable copies of each real planning basis show the old selector returning no recovery after
spending its 32,768 preparation allowance. The identical default forecast succeeds under the
full allowance at 41,524 work with Oil and 41,124 without it. Forecast construction alone
therefore cannot fit the old sub-budget; common preparation exits before pulse fallback.
The first successful selector basis is 972 and its pulse is applied at 973. Both episodes
deliver exactly 594 jobs without worker or admission failures. This distinguishes delivered
work from useful movement, and rules out missing-origin and Oil-only explanations.

These records establish the old failure and its preparation bottleneck, not complete success
of any alternative allocation. A repair needs actual survival through bomb flames and must
retain the shared work ceiling, contact/pit checks and archived policy behavior.
