# Teleporter v0.3.42 integration addendum

Based on committed v40 `/tmp/grenadiers-stopwatch` at `4aafbd49d3209c32793c64ef9b337693db4ba49b` and the adopted Teleporter/Invisible Bombs packages. This preserves the accepted Teleporter behavior.

The architect inspected the active `/tmp/grenadiers-invisible-bombs` tree read-only while its domain changes were partly present and its catalog/projection policy remained incomplete. Those observations are provisional, not findings against v41. Integrate against reviewed v41 without interrupting its author. The architecture task ran no tests, builds, native sessions or CI. Root persisted this returned addendum.

## Existing seams

| Implementation | Integration responsibility |
| --- | --- |
| `core/src/game/simulation.rs:172` | Add private `State.last_teleport_tick: Option<Tick>` initialized to `None` |
| `simulation_pickups.rs:9` | Return bounded deferred requests; commit movement after successful canonical pickup traversal |
| `simulation_step.rs:38` | Preserve Stopwatch preflight/rollback; resolve deferred teleport after pickup-error handling |
| `movement.rs:142` | Reuse `position_is_free` for geometry, adding Contact and fire checks |
| `simulation/contacts.rs:6` | Never call Contact grace initialization on teleport |
| `protocol/v8/projection_facts.rs:11` | Extend private `WorldFacts` without replacing live/event Timed provenance |
| `v8/projection_capture.rs:23` | Reuse explicit-catalog capture and capture marker unconditionally |
| `v8/projection_event_validation.rs:143` | Preserve exact pause arithmetic and capability guards |
| `v8/projection_binding.rs:19` | Add marker capability checks; retain fuse/phase binding |
| `v9/projection_recipient.rs` | Share narrow pure helpers; create v10-specific actor/state assembly |
| `web/src/game/v8/semantics.ts:111` | Reuse parameterized semantics with fixed version catalogs |
| `web/src/game/client/client.ts:194,232` | Check marker regression after freshness checks, before mutation |
| `web/src/game/arena/scene.ts:251` | Extend existing actor-motion lifecycle with marker-change snap |

## Domain: deferred collection and persistent marker

Own domain/catalog/generation changes and focused core tests. Preserve v41 audiences and Stopwatch behavior. Have `collect_pickups` return bounded requests such as `[bool; 2]`: each actor occupies one pickup cell, so at most two requests occur. Handle Teleporter in the collection coordinator, not equipment, weapon or timed-effect application.

During canonical traversal, select collectors using existing distance and assignment priority; consume eligible Teleporters and emit ordinary `PickupCollected`; record collectors without moving them; continue every ordinary and fallible timed pickup. Capture pickup audiences under Cloak policy at emission; Invisible Bombs' bomb-event audience intersection does not apply.

After successful collection, call focused `simulation_teleport.rs` before incrementing running updates and deciding results. Choose at most one request by assignment priority. Both requests may be consumed, but perform one swap attempt, without fallback, arrival collection, additional action phase or second explosion plan.

Two-player target selection is a singleton with no RNG draw. A living disconnected opponent is eligible. Equal positions, absent/dead target or either unsafe destination consume the pickup without changing positions, residuals, turn intent, grace or marker.

Prepare both exact fixed-point destinations and grace sets before mutating either actor:

1. Derive valid existing grace from that actor's stored IDs, existing bombs and positive overlap at its current position.
2. Check bounds, hard/soft walls and solid Timed/Remote bombs with `movement::position_is_free`.
3. Check Contact bombs separately because `blocking_cells` omits them. Reject overlap unless the arriver already has valid grace for that exact mine.
4. Reject positive overlap with active fire regardless of Shield, Heart or damage grace.
5. Retain only the arriver's valid prior grace still overlapping its destination.

Never transfer another actor's grace, grant new grace or call `contacts::initialize_grace`. Tangency remains non-overlap; actors remain nonblocking.

Commit both positions, retained grace, cleared residual/pending-turn state and `last_teleport_tick = next_tick` together. Preserve identity, heading, held-direction recency, effects, inventory, ownership and input high-water marks. Stopwatch freeze does not prevent involuntary teleport.

Do not append a movement segment or rerun Contact/fire checks along the jump. Existing traces describe pre-teleport movement. Check destinations against the final world after placement, explosion, fire damage and ordinary pickup application.

The marker survives movement, effect expiry, death, disconnect/resume and results; reset only with new `State`. Derived `Clone`/`Eq` covers rollback/replay. Existing pre-mutation rollback for possible timed-pickup overflow must restore preceding Teleporter consumption, pause provenance, movement, inputs, inventory, bombs, pickups and all other state. Deferred teleport adds no fallible mutation after collection succeeds.

## Protocol v10 and pinned archives

Create version-specific v10 state/server/scalars/module wrappers, admission and recipient assembly, `api/gameplay-v10.schema.json` and generated `web/src/game/v10/schema.ts`. Visible actors require `lastTeleportTick: Tick | null`, including explicit null, and retain `voluntaryActionsFrozen`. Hidden actors remain exactly `{type,id,seat,alive}`. Do not change archived v7/v8/v9 actor DTOs or v9 schema.

Reuse unchanged command/settings/room/bomb/event/inventory DTOs. Teleporter adds no action, effect, inventory slot, event variant or replay operation. Explicit catalogs are v8's pinned 14 unpaused items, v9's final reviewed v41 16 items, and v10's 17 items. Never derive archives from domain `ALL` or call v9 public wrappers and patch their output. V10 uses shared capture/bind/settings/room helpers with its own catalog.

Add one private marker to shared `WorldFacts`; capture it regardless of requested protocol so archives can reject unsupported history. Before constructing `ValidatedWorld`, non-null requires Teleporter supported and enabled, checked `round_start = physical_tick - running_updates`, and `round_start < marker <= physical_tick`. Null remains valid after failed consumption; dead actors may retain a marker. Use physical time, never later room/lifecycle result-hold time. One private marker ensures equal values on both full actors; copy only to visible actors. Test equality and exact hidden omission.

Add capability validation to binding while retaining `state_binding` and `completed_binding`. Archives v9/v8 reject Teleporter settings/catalogs, exposed pickups, historical pickup events absent from the final world or hidden from recipients, and non-null marker even if a malformed state's mask was cleared. Extend domain-to-wire conversion; reserved `projection_items::pickup_item` already maps Teleporter and existing event validation supplies historical enabled-item checks.

Preserve `WorldFacts::timed_bombs`, `EventFacts::timed_bomb`, checked original-fuse/pause arithmetic, live pause consistency and archived resumed-history rejection. Share only pure freeze calculation and captured-audience filtering/reindexing helpers; versions retain their own assembly/encoding. Do not copy the validator or broaden archived actor types.

Carry reviewed v41 semantics: four audiences are valid for bomb events only under enabled policy; other event restrictions stay fixed. Validate complete bombs and all events, including `Neither`, before filtering. Preserve paired encoding before revision advancement, report clearing or admission commit.

## Production migration and replay

Switch all current production routes to v10: server game socket/connection/dispatcher; rooms profile/default settings; match-runtime publication/runtime/admission; match-owner types/driver admissions; current credential/type imports and runtime/replay helpers representing production.

Keep publication capture → bind → prepare both frames. Admission's event-free body and separate notice both use v10 with the same retained marker before commit. Cover join, resume, replacement, ordinary snapshots, result holds, retirement/forfeit and shutdown; no archived fallback or private-fact serialization.

Keep rules 5, generator 2 and private replay format 2. Journals reconstruct swaps and markers from settings/accepted operations without RNG changes; retain old masks and record/journal bounds. Authenticate current clients as v10 and explicitly reject v9 handshakes while keeping archived tests. This is a paired client/server release, with new rooms/reload and no persistence migration or automatic live-room conversion.

## Browser validation and authoritative snap

Add v10 validation with fixed 17-item catalog. Reuse `expandedProfileSemanticErrorFor(..., catalog, true)`; preserve v8 14/unpaused and archived v9 final-v41 16-item wrappers. Extract the small freeze check rather than copying v9 validation. Preserve authenticated-recipient blindness and Cloak checks in `clientProtocol.ts`.

Stateless validation requires nullable safe Tick, no field on hidden actors, enabled Teleporter for non-null, checked physical-round bounds, equal markers when both actors visible, and unchanged freeze/effect/pause/catalog semantics.

Add a small cross-frame guard outside nearly full `client.ts`, called by `acceptAdmitted`/`acceptSnapshot` after sequence/generation/freshness checks and before state changes. For matching actors visible in both accepted states within the same room/round, null→non-null and equal/increasing values are valid; non-null→null/lower is invalid. New round/actor and newly revealed actors initialize normally. Ignore stale frames first. Same-round accepted reconnect must not reset known history.

Carry marker through `presentSnapshot` and arena types. Extend existing per-actor motion with last marker, not a separate lifecycle cache. A retained actor's changed marker sets both interpolation endpoints and draw position to authority, suppresses jump walking, preserves freeze-entry snap, then resumes interpolation on later unchanged markers. New/revealed/admitted actors initialize without historical cue; cleanup/reset removes marker. Support short swaps, hidden initiators, event-free admission and coalesced swaps returning to prior position; no distance or pickup-event heuristic.

Add distinct pickup art and EN/PL safe-landing consumption text. Preserve captured-event pickup audio even for unsafe consumption. No snapshot-difference sound, origin trail, hidden-position cue or new countdown.

Migrate current client/protocol/types, GamePanel/settings and test support; maximum current settings count 17 retains canonical membership. Register v10 alongside archives in `web/scripts/{generate-gameplay,check-gameplay,gameplay-generator.test}.mjs`. Migrate production E2E schemas/hardcoded authentication including game/reconnect/latency/browser diagnostics; explicitly archived fixtures retain old versions.

## Catalog and acceptance

Teleporter stays bit 21 (`2097152`); Oil bit 20 stays disabled. Supported union `3145712`, 17 items and 131,072 exhaustive masks. Append weight 15 after Invisible Bombs: pool 350→365, interval `[350,365)`, sole-item pool 115. Preserve every existing 65,536-mask assertion, map, pickup layout, RNG/replay case. Measure actual gate cost before independently reviewed batching; no sampling, coverage-floor changes or timeout increases.

Use separate independent domain/rollback and protocol/privacy/browser reviews. Distinguishing cases:

- Fractional/equal swap, both priorities, two consumed/one attempt, no arrival pickup/action; second unsafe landing leaves no partial position/grace/marker mutation.
- Actor-specific solid/Contact grace, stale/source-only grace rejection, protected-fire rejection and final-world occupancy.
- Frozen actors, Reverse/Slow held input, Cloak/Invisible Bombs, living disconnected/dead opponent, result persistence and new-state reset.
- Late timed-pickup overflow after Teleporter consumption restores full state.
- Marker at round start/first running/current/future tick, result lifecycle clock and malformed disabled history.
- Archived live/event-only/history rejection, unchanged private Timed provenance and `Neither` validation.
- Hidden omission, malformed hidden-world facts before filtering and atomic second-frame preparation failure.
- Short/coalesced/hidden/event-free snaps, stale/regressing frames and reconnect/reveal without replayed audio.
- Full deterministic State/reports/both-perspective replay equality.
- Both maximum v10 frames below 64 KiB; unchanged 96 events, 16 bombs, 195 flames, ≤64-byte records and <4 MiB journal bounds.

The adopted complete ordinary seed-31 pure-core route is prerequisite to native work. Keep the single keyboard-only fixture proving safe swap, reconnect and unsafe consumption. No injection, seed search, retries or timeout weakening.

No architectural blocker remains. Re-read reviewed v41 catalogs/audiences/recipient guards before implementation. Principal risks are archived capture discarding history, general free-position checks bypassing Contact safety, swaps during collection traversal, and regression validation preceding stale-frame rejection.
