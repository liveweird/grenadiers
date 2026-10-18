# Gameplay v7: recipient state and manual actions

V7 is the live gameplay contract. The closed schema is `api/gameplay-v7.schema.json`;
generated browser types are `web/src/game/v7/schema.ts`, with the active client in
`web/src/game/client`. Server and browser migrate together; there is no live v6 fallback.

Current Classic rooms advertise empty available/enabled item sets. Optional pickups and
nonempty item settings are rejected. Timed/Standard bombs and existing three upgrades remain
the playable loadout; this migration does not activate equipment, buffs, curses or special weapons.

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
succeeds, the semantic helper checks that each ordinary effect occurs at most once, at
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
no revision advance or report clearing. Live routing, admission, resume and browser handling
are integrated in the live owner/socket transaction. Every retained or closing notice selects
the authenticated seat perspective; personal admissions are preencoded before ownership or
replacement commits. Projection failure advances no revision and clears no reports.

## Frame bounds

A serialization-based conservative proof measures every closed union branch and fills arrays
to their schema maxima, relaxing uniqueness and cross-object constraints to overapproximate
valid messages. Snapshot is at most 61,586 bytes and personal admission 36,472 bytes, below
the unchanged 65,536-byte ceiling. A separate schema-valid representative measures 60,400
and 35,382 bytes. Encoding still rejects oversize frames without truncation.


## Compatibility and unsupported equipment

Reload older clients after upgrading. Version 6 is rejected before a credential is claimed, so
the same credential can still authenticate using version 7. Successful admission restores the
shared manual action floor as `lastActionId` without replaying historical events or audio.

`manualAction` with `action: "bomb"` retains ordinary bomb behavior. Equipment is not available
yet: `action: "equipment"` receives correlated `invalidMessage`. Only its connection sequence
is consumed; manual high-water, mailbox allowance and journal remain unchanged. A later bomb
may use that action ID with a fresh sequence.

The client validates structural and semantic input before updating state, requires its own
actor to be visible, and removes hidden actors/bombs from presentation rather than fabricating
positions or fuse deadlines. World interpolation, effects and audio use `physicalTick`;
lifecycle countdowns use the independent room tick. Inventory, effects and grace have static
EN/PL status labels without replaying hidden spatial cues or announcing every timer update.
