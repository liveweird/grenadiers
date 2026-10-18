# Teleporter — v0.3.42 implementation package

Coordinator-adopted package from independent architect
`/root/remaining_curse_architecture`, Remote baseline
`e01d32ccba67e3390afcc906f0eaebedc0f56f90`. Depends on InvisibleBombs41 and
approved Landmine/Cloak/v9/Stopwatch packages. Plan only; not implemented/tested.
Keep rules5/generator2/journal2; introduce gameplay-v10. Do not broaden v39/v9
foundation. No user decision blocks the chosen documented safety policy.

## Immediate, deterministic behavior

Teleporter is a consumed immediate pickup, not equipment/weapon/timed effect.
Collector and target must be alive at pickup phase. Current two-player game has
one eligible enemy: a singleton selection needs no RNG draw/state/journal entry.
A living disconnected actor remains eligible; dead/missing target consumes
without swap. No claimed multi-player/monster targeting; future multi-seat
selection needs separate deterministic versioned design.

Swap exact fixed-point positions. Identity/seat/heading/upgrades/inventory/effects/
grace/ownership/capacity/input high-water/connection and held direction recency
stay with each actor. Clear both movement residual and pending turn intent only
on successful non-no-op swap. Equal positions or unsafe/no-target consumption
leave all movement state/grace/markers unchanged. Current held input resumes
next update under Reverse/Slow/Stopwatch. Teleport is involuntary and may move
frozen actors without thawing. Explain safe-landings-only consumption in EN/PL.

## Deferred collection and phase ordering

Current pickup traversal mutates based on live actor positions, so swapping
inside traversal would make outcome depend on cell order and collect on arrival.
Preserve existing step: expiry/pause -> input -> voluntary movement/Contact ->
old-fire damage -> manual/Diarrhea -> immutable explosion -> final-fire damage.
Then traverse pickups in canonical cell order, applying ordinary pickups and
events normally; consume Teleporters and record at most two bounded requests
without moving actors. Finish ALL fallible timed-pickup applications, choose at
most one request by alternating RoundAssignment priority, check both landings
against final post-explosion world, and atomically commit or do nothing.

Two actors collecting distinct Teleporters consume both but cause one swap;
same-cell contention keeps existing distance/tie rule. Never rescan after swap,
collect an arrival pickup, retry a failed request, swap back or run another
action/Diarrhea phase. Earlier ordinary pickups remain with their collector;
event audiences remain captured at emission, including newly applied Cloak.
Round result is decided normally after this phase.

## Safety, contact and rollback

For BOTH arriving actors reuse fixed-point footprint/free-position/positive-area
overlap helpers and Landmine passability. Entire destination must satisfy arena/
hard-wall/soft-block legality. Reject overlapping solid Timed/Remote bomb unless
the arriving actor ALREADY has valid exit grace for that exact bomb; reject
Contact mine unless that actor already has initial-overlap grace for that mine.
Reject active fire even with Shield/Heart/damage grace. Tangency is not overlap.
Retain only actor's own prior grace for existing bombs overlapping destination;
never transfer other actor's grace, grant landing grace or retain source-only
grace. Preserve nonblocking actor overlap and atomic participant departure.

Final-world checks include newly planted bombs, fresh fire and removed bombs.
Compute both landings and retained grace before either mutation. Teleport has
NO swept path: no intermediate walls/fire/mines/pickups, no extra MovementTrace
segment/contact/damage pass. Existing traces describe earlier voluntary movement.
Already-graced destination overlap creates no new Contact entry. Coarse unsafe
landing clues under InvisibleBombs are intentional collision clues; no detailed
hidden destination reason is serialized.

Deferring after timed-pickup success avoids new fallible mutation after swapping.
A later timed pickup overflow after Teleporter consumption restores complete
pre-step state: consumption, pauses, inputs, movement, bombs/damage/events and
unchanged marker. A failed second destination check never half-swaps or clears
the first actor's movement/grace. Existing seed/assignment/settings/operations
determine replay; include marker in State equality/clone, add no randomness or
larger records. Test safe/failed/simultaneous/fractional/frozen/hidden/disconnected
routes, publication windows, resume/death/reset and full reports/State equality.

## Required gameplay-v10 displacement marker

Collection event is insufficient for snapping: Cloak can make it private, short
swaps resemble ordinary movement, event windows coalesce and admission has no
history. Add REQUIRED visible-actor `lastTeleportTick: Tick | null` as persistent
position epoch. No new event or counter/overflow. One private State
`last_teleport_tick: Option<Tick>` suffices because current two actors always
swap together. On actual successful non-no-op swap set validated current physical
step tick; persist through movement/expiry/death/resume/results, reset only on
new-round State. Hidden actor remains EXACT {type,id,seat,alive}, no marker.

Validate full canonical facts before filtering: non-null requires Teleporter
enabled, both full actor markers match, tick later than physical round start and
not beyond captured physical tick. Derive start via checked physicalTick/
runningUpdates arithmetic and captured result clock. Retain all existing world,
effect/bomb/provenance/identity/clock guards. Web checks equality only when both
actors visible; never fabricate hidden marker. Reject regression within same
room/round while allowing newly revealed actor and normal round reset.

Create version-specific v10 visible/state/envelope DTO/schema, reuse unchanged
command/room/item/bomb/event DTOs and narrow v9 helpers; no shared-v7 actor changes
or blanket validator copy. Production server/client/auth becomes10; explicit
version-mismatch and missing/null/invalid marker tests. Pin archivedv9 to v41
catalog/shape and reject Teleporter settings or actual history, including
event-only pickup records; preserve v8/v7/older guards.

## Renderer, art, audio

Carry marker through presentation. Changed marker on retained actor sets BOTH
interpolation endpoints and draw position to latest authority immediately, then
ordinary movement interpolates normally. New/revealed/admission actor already
snaps; initialize epoch without replayed cue. Preserve hidden-root/shadow/motion
removal and stale/admission generation guards. This handles hidden initiator,
short swap and multiple swaps between observations without global interpolation
removal or distance heuristic.

Distinct pickup and EN/PL instructions, no invented inventory/timer. Existing
PickupCollected audio uses Cloak audience, including unsafe/no-target consumption;
target may have no pickup sound by design. No snapshot-diff teleport audio. Any
optional destination-only cue must respect reduced motion and never replay on
admission/reveal; no origin trails or inferred hidden target location. Public
marker carries no other actor ID/origin/target/reason/effect detail.

## Catalog

Bit21=2097152; OilBarrel bit20 remains disabled. ALL1048560 ->3145712;
available16 ->17; masks65536 ->131072. Append weight15 after InvisibleBombs,
pool350 ->365 interval[350,365), sole pool115. Canonical wire order appends
teleporter after invisibleBombs, omitting oilBarrel. Preserve all old masks/maps/
pickup layouts/RNG; exhaustive131072 masks, deterministic sharding allowed but
no sampling/removing assertions.

## Ordered ownership and acceptance

1. Domain: enabled_items,pickups,simulation_pickups,simulation/observations,
   focused simulation_teleport module, shared movement/grace helpers only as
   necessary, generation/tests and ENTIRE pure-core native route first.
2. Protocol/runtime after marker agreement: api/gameplay-v10 schema/fixtures,
   v10 narrowly shared binding/validation/filtering, archived guards, production
   server socket/profile/match runtime+owner migration and replay tests.
3. Web after schema: generatedv10/types/semantics/guards/auth/clientProtocol/
   presentation/settings/GamePanel/arena scene+art/audio/matchingENPL.
4. Coordinator: native fixture/config/scenario/Compose/CI/version/topicdocs.
   Serialize shared files; independent domain ordering/rollback and protocol/
   privacy/rendering reviews, keep400-line ceiling.

Tests: off-center swap/identity/heading/loadout/ownership preservation;
success-only residual/turn/grace changes; held inputs/action floors; same-cell/
both priority assignments/two-consumed-one-swap/no-target/equal/no-retry;
both landing checks/bounds/walls/tangency/bombs/actor-specific grace/source-prune/
Contact/no-new-grace; fire despite protection/final-world occupancy; no swept
intermediate damage/contact/arrivalpickup/extraaction; frozen/Reverse/Slow/Cloak/
InvisibleBombs/disconnected/dead/results/reset; full late-overflow rollback;
complete-world marker bounds/equality before hiding/hidden-union leak/archived
history rejection; short/long/hidden/coalesced/event-free snap/stale/regression;
paired live/admission/resume/replacement/final atomic second-encode failure;
full replay and exhaustive masks. Preserve96events(no new event),16bombs,
195flames,<64KiBframes(remeasurebothmaxv10),<=64-byte records,<4MiB journal.

## Proposed normal native fixture — unexecuted

First prove entire generated seed31 sole-Teleporter115 route in core with actual
input operations, geometry3,1/11,11, legality/timing/survival. No seed search.
Prepare P1 ENdesktop/P2 PL320 reduced-motion before play.

1. P1 reveals3,1 by bomb2,1, escapes1,1->1,2, waits fire clear; P2 stays13,11.
   Both markers null.
2. P1 returns2,1 and HOLDS East until successful marker change, releasing in
   finally. Do not moveTo3,1: the swap prevents staying at intended destination.
3. Both streams show matching nonnull marker=collectiontick, opposite ends with
   identities/headings preserved, consumed pickup/no new inventory or timer.
   Fractional exactness is core proof; public positions can include later held
   movement. Compositor crops prove actors painted at authoritative destinations;
   scene unit tests separately prove first draw snaps without board traversal.
4. Normal reconnect retains marker/current positions, no historical/audio replay.
5. Bounded unsafe case: P1 reveals mirrored11,11 with bomb12,11, escapes
  13,11->13,10 and waits clear. StageP1at12,11. P2 plantsTimed at current3,1
   and briefly stays in own exit grace; P1 immediately enters11,11. Pickup
   consumed but no swap/markerchange because P1 lacks P2bomb grace. P2 escapes
  2,1->1,1->1,2 BEFORE screenshots. Core proves timing/footprints/survival.

Keyboard/ordinaryUI/read-only observations/existing reconnect only. Release
keys finally; no injection/seed search/retries/timeout or assertion weakening.
Run all/coverage/security/k8s unchanged gates, then independent reviews, fresh
cumulative same-image Compose/K8s including audio, final CI and actual evidence.
Use corrected preflight with pinnedNode/YAML/exactsuccessstdout/failfast and
pre/postexternaldependencyhashes. Deploy pairedv10; explicit old-client handshake
failure, oldmask/journalreject. Rollback precedingpairedv9/newrooms/reload; no
persistent-data migration.
