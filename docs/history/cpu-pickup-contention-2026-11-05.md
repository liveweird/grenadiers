# Hard CPU pickup contention — 2026-11-05

The [backlog](../../BACKLOG.md#single-player) owns delivery status. The implemented contract
belongs in [CPU contracts](../game/cpu-contracts.md#hard-pickup-contention).

## Controlled physical comparison

The paired queued-owner test uses an explicitly curated public scene, not a generated arena
or a restored concealed-content seed. Both policies see the same terrain, actors and two
exposed Blast Range upgrades. The CPU starts at `(7,7)`; upgrades A and B are at `(5,7)` and
`(9,7)`. The human rival starts at `(13,7)` and legally moves west, with held-input refreshes.
Both upgrades are two cells from the CPU; the rival is eight cells from A and four from B.

Hard65's stable baseline chooses A. Its CPU physically collects A and the human collects B.
Hard66 chooses B, physically collects it while A remains available, then makes further
settled movement. Assertions bind the actual collector identity and seat, pickup kind and cell,
rather than counting a worker proposal as collection. Both cases retain bounded queue/work
and failure checks. This demonstrates the specified tie decision, not a general strength or
win-rate claim.

The recorded trace collects A for Hard65 at tick 238 and B for the human at tick 251; Hard66
collects B at tick 238. Both deliver 13 queued jobs with zero failures. Maximum charged work is
2,950 for Hard65 and 3,267 for Hard66, below the unchanged 65,536-unit allowance.

The separate six-case generated matrix covers three board sizes with lairs off/on. Its
Hard65 and Hard66 plant, destruction, survival, elimination, stall, completion, admission and
first-plant metrics match. It detects no measured regression in those cases; it does not
demonstrate broader tactical improvement.

## Review corrections

Rival navigation must describe physical passage, not the AI's preferred pickup transit rules.
The initial comparison is restricted to exposed capacity/range upgrades; an unsupported pickup
anywhere declines the optional comparison. A shared charged physical mask avoids repeated
linear soft-block/Oil scans at every flood-fill neighbor. Burning Oil remains optimistically
passable rather than overestimating a shielded rival's distance.

Independent review also found that parameterizing Hard65's fallback had bypassed its
ContactV2 observation gate. Restoring the original guard and testing its exact preflight work
and zero RNG draws preserves the archive; the new policy retains that capability boundary.
