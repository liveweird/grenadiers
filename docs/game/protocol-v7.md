# Gameplay v7: dormant physical items contract

V7 freezes the wire representation needed for expanded physical items while the live gameplay
endpoint remains on v6. The complete closed contract is `api/gameplay-v7.schema.json`; generated
browser types are checked in at `web/src/game/v7/schema.ts`. This version does not select v7 at
runtime. Canonical and per-recipient projection helpers are staged below; live routing and
item activation remain separate increments.

Successful authentication is fixed to protocol version 7. Authentication input remains the
bounded cross-version integer. The v7 `manualAction` command replaces `placeBomb` and carries the
existing sequence, room, round and action fences plus `action: "bomb" | "equipment"`. Personal
admission reports the same required nullable counter as `lastActionId`.

## Physical state

`GameState` adds `physicalTick`. An actor is exactly one of these closed shapes:

- `visible`, containing all v6 actor fields plus `inventory`, `effects` and
  `damageGraceRemainingTicks` (0–60);
- `hidden`, containing only `type`, `id`, `seat` and `alive`.

Inventory has nullable equipment (`kick`, `punch`, `bracelet`, `wallPass`, `heart`) and nullable
weapons (`remoteBomb`, `piercingBomb`, `powerBomb`, `landmine`). An actor carries at most 16 timed
effects. `shield` lasts 1–300 ticks, `stopwatch` lasts 1–180, and `invisibilityCloak`,
`goldenFlame`, `shortFuse`, `diarrhea`, `slowFoot`, `shortFlame`, `longFuse`, `reverseControls`
and `invisibleBombs` last 1–600.

A bomb retains `id`, `cell`, `owner`, `range` and `placedAt`. It replaces `detonatesAt` with one
closed trigger: timed (`remainingTicks` 0–600 and `paused`), remote, or contact. Its `blast` is
`standard`, `piercing` or `power`.

The event window grows to 96 entries with indexes 0–95. Existing event variants carry the new
bomb shape. `damagePrevented` adds actor `id`, `seat` and source `heart` or `shield`.

## Items and validation levels

Pickup kinds preserve `bombCapacity`, `blastRange` and `movementSpeed`, then add these 22 IDs in
stable mask order:

1. `kick`
2. `punch`
3. `bracelet`
4. `wallPass`
5. `heart`
6. `remoteBomb`
7. `piercingBomb`
8. `powerBomb`
9. `landmine`
10. `shield`
11. `stopwatch`
12. `invisibilityCloak`
13. `goldenFlame`
14. `shortFuse`
15. `diarrhea`
16. `slowFoot`
17. `shortFlame`
18. `longFuse`
19. `reverseControls`
20. `invisibleBombs`
21. `oilBarrel`
22. `teleporter`

Rooms publish `availableItems`; settings add `enabledItems`. Both are unique arrays of at most 22
known IDs. The other four settings fields keep their v6 bounds.

Validation intentionally has two levels. JSON Schema/AJV checks closed shapes, required nulls,
unions, scalar and timer ranges, array bounds, known IDs and item-array uniqueness. After AJV
succeeds, the dormant semantic helper checks that each ordinary effect occurs at most once, at
most six independent `slowFoot` effects occur (equal durations are valid), and every enabled item
belongs to the containing room's available set. Raw AJV cannot prove these cross-entry and
room-context rules. The shared valid-input semantic cases live in
`api/fixtures/gameplay-v7-semantics.json` for Rust and browser parity.

Rust deserializes incoming client messages and checked values; server DTOs are serialization-only.
The canonical projection increment adds the stronger boundaries described below.

## Canonical observations and recipient projections

Validate the complete physical world and unfiltered report window before hiding anything.
Check actor identities, effect semantics, arena geometry, bomb ownership and capacity,
timestamps, results and event bounds. Contact bombs count against ownership limits. A malformed
record fails validation even when that recipient would never see it.

Opaque validated observations do not certify a playable rules profile. Future-item fixtures
can exercise cloak, bomb blindness and untimed triggers through pure projections. Separate
Classic room binding rejects all optional inventory, effects, grace, pickups, special triggers,
blast modes, non-public audiences and nonempty item settings. Matching available/enabled arrays
cannot make an unsupported Classic world valid. Expanded identities are introduced with activation.

Observe effects, Heart grace and bomb fuses at the physical simulation tick. Frozen result
bodies do not age with room time. Event bomb fuses use their event tick; a chain detonation
may still have positive fuse remaining. Read-only capture adds no State injection interface.

An owner always receives their complete actor. Cloak removes the rival's spatial actor state;
bomb blindness removes every bomb record, including owned bombs. Owned counts remain complete
and fire remains visible. Event audiences are captured at emission, before later expiry,
elimination or reveal can change visibility. Filter those captured audiences before assigning
contiguous recipient-local indexes within each tick.

Reject nonconsecutive reports, more than three reports or 96 unfiltered events without truncation.
Prepare both recipient encodings before returning a pair; failure returns neither and performs
no revision advance or report clearing. Live routing, admission, resume and browser migration
remain a separate increment. Until that migration, the endpoint continues to use v6.

## Frame bounds

A serialization-based conservative proof measures every closed union branch and fills arrays
to their schema maxima, relaxing uniqueness and cross-object constraints to overapproximate
valid messages. Snapshot is at most 61,586 bytes and personal admission 36,472 bytes, below
the unchanged 65,536-byte ceiling. A separate schema-valid representative measures 60,400
and 35,382 bytes. Encoding still rejects oversize frames without truncation.
