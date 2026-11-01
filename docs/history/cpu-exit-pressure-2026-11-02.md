# CPU firing-position comparison — 2026-11-02

This investigation records the frozen Hard57 baseline and controlled Hard58 comparison. It
does not claim a restored match or establish a guaranteed kill against a reactive opponent.
Work status belongs in [the backlog](../../BACKLOG.md#single-player).

## Equal-distance scene

All item switches are enabled. CPU Two starts at `(5,5)` and naturally collects BlastRange,
raising range from two to three. Rival One is centered at `(7,7)`; the sole soft block is
`(7,8)`. Cloud, lairs and spike pits are disabled to isolate firing-position ranking.

North `(7,5)` and west `(5,7)` are equally close. North covers one currently open adjacent
terrain branch; west covers two. Hard57 selects north by its existing coordinate tie-break.
This measures present branch coverage, not future rival movement or escape safety.

Core `exit_pressure_witness_tests` preserves the real pickup, north/East choice and exact
21,203 charged work. Actual game walks to either origin produce complete useful plant
certificates: placement tick49, detonation229 and survival259. The separate complete origin
queries cost3,981 north and3,977 west, including usefulness verification. These are separate
queries, not an aggregate candidate-search cost or a mob-contact lease.

## Queued owner witness

The real four-seat owner has human One/Three/Four and CPU Two. Extra humans are distant and
keep the round running after One dies. The controlled scene is installed without changing
runtime rules, clocks, assignment or actor identities. Pickup collection occurs in owner
physics; the test captures an actual queued observation, settled Stop, RNG and hazard history.
A diagnostic replay matches the worker's returned intent and work.

At actual queued tick183/Stop183, Hard57 chooses north/East for21,691 total work. It plants
at232, detonates412 and survives the complete flame horizon442. The idle rival dies at412;
this is an execution witness, not a competitive win-rate test. Worker delivery succeeds and
the recorded admission categories are zero.

An environment-enabled control deliberately remains separate. Its pits at `(6,5)` and nearby
sites make north farther in the existing navigation graph; Hard57 already chooses west for
55,822 work and plants/survives on the same ticks. Improving that choice cannot be claimed as
an effect of future exit ranking.

The test-only common owner builder preserves the existing environment-enabled Teleporter and
farming scenarios. Local diagnostic logs use `target/quality-review/exit-pressure-*`.

## Diagnostic validation

Formatting, strict workspace Clippy and the repository gate pass. Debug and release each pass
three Core witnesses, two exit-pressure owner cases, two legacy late-Teleporter owner cases
and the corner-farming owner case. Independent review is clear after requiring an empty
hazard map after the flame horizon and observing the rival cell’s exact flame expiry.

## Hard58 owner comparison

The same actual queued scene now selects west `(5,7)` with South movement. It spends21,876
work versus archived Hard57’s21,691:185 additional logical units in this controlled query,
not a general latency measurement. The first real bomb remains232/412/442 for placement,
detonation and complete flame horizon. At detonation the actual hazard map covers both open
lateral branches `(6,7)` and `(8,7)`, rather than only north `(7,6)`. CPU Two survives the
whole500-update episode; the stationary rival dies at412 in both versions.

The environment-enabled control runs both57 and58 and preserves west, actual planting and
full flame survival. Replay29 closes the new56/58 pairing; replay28 and Hard57 remain archived.
An integration test caught and corrected a no-lair dispatch collapse: the new internal cautious
and farm variants preserve their different farm-origin flags while both enable exit ranking.
