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

Do not remove those guards without a replacement proof. Intentional collection would need to
cover every possible live partner, both endpoints, retained motion and post-teleport mob/hazard
contacts without observing private partner-selection randomness. Destruction would need to
cover collection throughout the certified bomb escape, not merely show that the eventual blast
reaches the pickup.

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
