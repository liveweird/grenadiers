# Punch — v0.3.45 implementation package

Adopted coordinator summary of `/root/remaining_curse_architecture`'s read-only
design. Inspected Remote e01d32ccba67e3390afcc906f0eaebedc0f56f90; depends on
Landmine37 through Kick44 packages. Plan only, not executed acceptance evidence.
Keep gameplay-v10, rules5, generator2, journal2. Oil and Bracelet remain disabled.

## Existing contract and equipment admission

The existing DTO reserves ManualActionKind::Equipment (v8/client.rs); dispatcher
currently rejects it. Existing mailbox has shared action high-water and eight
commands per pending batch. Activate persistent replaceable Punch with existing
Equipment command plus core UseEquipment/mailbox/runtime/private replay variants.
Require enabled Punch, equipped Punch, alive, voluntary actions unfrozen and
elapsed cooldown. Pickup follows actions, so newly acquired gear acts next update.

Admit Equipment ONLY in running Expanded settings enabling Punch. Client sends
none otherwise; server rejects unsupported configuration BEFORE mailbox/ID
admission. With Punch enabled, wrong gear/cooldown/freeze/no target/blocked landing
are admitted gameplay no-ops consuming the SHARED Bomb/Equipment ID high-water.
Preserve duplicate-before-budget ordering, Counter bounds/error precedence,
eight-command pending-batch allowance across movement/manuals, independent core64
logical-input bound, movement-first drain, raw Clear cancellation, server Clear/
TTL preserving accepted manual actions, fences dropping queues but retaining
high-water, new-round reset and resume restoration. Frozen actions drain/drop,
never queue for thaw. Unsupported raw core Equipment fails before mutation after
existing input/tick/bomb validation priorities. No-Punch adapter stays unchanged.

Replay validates the EFFECTIVE settings mask at each operation, including lobby
settings updates. Initial header settings alone are insufficient. Old masks plus
Equipment stay invalid; new enabled-Punch mask is the discriminator authorizing
ReplayAction::Equipment without changing journal format or public version.

## Bounded ordinal manual waves

Punch-enabled rounds process the two retained per-seat queues in ordinal waves,
one next action per seat per wave. Only FIRST Bomb command per seat has gameplay
effect; later admitted Bomb commands drain with no retry/trigger/placement. Failed
Equipment may be followed by another attempt; success starts cooldown so <=1
success/actor/update, <=2 overall. Bounded buffers, no general unbounded scheduler.

Within each immutable wave:
1. First-Bomb Remote requests reserve owned armed Remote IDs.
2. Evaluate and resolve Equipment/Punch requests.
3. Process first-Bomb placements not classified as Remote triggers.

Remote reservations persist ALL LATER waves and prohibit subsequent Punch of
that ID. Punch then Bomb detonates at new cell; Bomb then Punch leaves accepted
trigger location fixed; same-wave Remote wins. One explosion plan runs later.

Before waves classify each alive seat's eventual placement origin: first unfrozen
Bomb unless it owns a Remote, otherwise active Diarrhea. An unfrozen first Bomb
suppresses Diarrhea even on failure; Equipment-only does not. Frozen manuals
drop first, so involuntary Diarrhea continues. Automatic attempts follow waves.

Preserve whole-update shared-cell placement veto: both eligible origins at same
actor-center cell, with alive/capacity/floor/no-soft conditions, both fail even
if Punch later clears current bomb occupancy. Owned Remote is trigger, not
claimant; capacity-ineligible seat cannot veto rival. This prevents queue padding
with Equipment from converting simultaneous placement conflict into one winner.
No world reservation/fallback; actual occupancy is checked at placement's wave.

## Facing target and three-cell wrapped search

Target one cell adjacent to current actor CENTER cell, in authoritative heading
after voluntary movement. Target selection does not wrap. Reverse already affects
movement-derived heading; Equipment does not invert again. Pending turn keeps
previous heading. No client target, nearest search or collision requirement.

Target Timed/Remote Standard/Piercing/Power, including sliding. Exclude Contact.
Reject source touching active fire, Timed naturally due AFTER Stopwatch pause
preflight, or previously/same-wave Remote-reserved ID; leave for later plan.

For source axis s, direction d, interior length n, candidate k is:
`1 + floor_mod((s - 1) + d * 3 * k, n)`.
Horizontal1..13, vertical1..11; <=n/gcd(n,3) attempts (13/11 now). Returning to
original cell fails unchanged. Landing needs interior floor/no soft/no other bomb
(including Contact)/no positive-area full-cell overlap with ANY living footprint.
Tangency okay. Protection/Cloak/Invisible/freeze/WallPass/bomb grace never waive
occupancy; pickups neither block nor collect. Oil support waits for Oil47.

Evaluate both candidates from SAME immutable wave world: same-bomb valid claims
choose RoundAssignment priority; distinct bombs same empty landing both fail;
other original source cells stay occupied even if moving; no next-landing retry
after inter-claim conflict. Commit accepted distinct moves together.

Logical hop is instantaneous, no intermediate swept path, damage, fire or mine
entry, including wrap. Landing on fire is legal and seeds same later explosion
plan. A burning landing cannot be punched away in a later wave.

## Cooldown, private facts and atomicity

Success at T blocks throughT+11, allowsT+12. Store private last_successful_punch_tick
and checked elapsed comparison (no added overflow deadline). Validate against
physical round clock. Retain across replacement/resume/death/results as history;
new round resets. Stopwatch advances cooldown but independently freezes action.
Failure changes neither bomb nor cooldown.

Success changes cell, stops active Kick motion, retains hasBeenKicked history and
sets hasBeenPunched; preserves ID/owner/range/blast/placedAt/captured fuse/trigger/
pause provenance. Clear moved ID from ALL actors' bomb grace, grant none. No
capacity release, equipment/weapon consumption, new ID or placement event.
Historical placement retains old cell; live/event-only detonation retains private
Punch history even if moved and detonated in same publication window.

Phases: existing validation/pause preflight; expiry/reveal; preexisting Kick motion;
inputs/voluntary moves/Contact/Kick claims; resolve Kick; old-fire damage; manual
waves/Diarrhea; ONE immutable explosion plan over final cells; final damage;
pickup/deferred Teleporter/result. Bracelet later inserts forced motion before
old-fire damage. Earlier-wave placements may affect later Equipment. No extra
input/movement/pickup pass. Teleporter checks current cells and own grace.

Late pickup overflow rolls back queues' effects, inputs/movement/cooldown/bomb
cells+history+motion/grace/pause/allocator/damage/pickups/events/teleport. Preserve
old validation/error ordering, Standard/Piercing/Power/Contact invariants.

## Projection, paint, controls and privacy

No new wire fields/events: existing inventory and current bomb cell suffice.
Validate all private cooldown/history/enabled/bomb/fuse facts BEFORE filtering,
including hidden/event-only facts and stopped Kick history. Archivedv9/v8/earlier
remain pinned and reject actual new settings/gear/pickups/cooldown/bomb history.
Both encodings precede publication/admission commit/revision/report clearing.

Use Kick's same-root immediate authoritative cell update. No second placement,
airborne collision location/intermediate interpolation/origin shadow. Fixed rule
HUD only: equipment key, 0.2s after successful punch. Cooldown private: no dynamic
ready/cooling flash, target outline, blocked reason or success sound exposing
InvisibleBomb targets. Public cell movement remains permitted inference. Existing
Cloak/Invisible event audiences and quiet reveal/resume stay intact; no audio
from snapshot differences. Preserve320px/reduced motion/teleportsnap/teardown.

Add remappable `equipment` defaultKeyE, preserve SpaceBomb and shared next action
ID. Running/alive/admitted plus Punch-enabled guard; no local frozen-action queue.
Prevent repeat; track physical-held keys PER manual action so Bomb/Equipment and
alternate bindings don't spuriously suppress/retrigger. Blur/visibility/disable/
remap/teardown clear held state; held remap requires key-up before activation.
Preserve movement priority/heartbeats/focus/global duplicate rejection.

Preferences migrate EXACT five-action v1 to six-action v2/new storage key:
prefer validv2; otherwise parse frozenv1 and retain ALL bindings, assign Equipment
E if free else first free from documented E,F,Q,then remaining supported keys.
Old <=10 assigned physical keys ensures free key. Display ACTUAL assigned key,
preserve oldv1 storage for rollback, savev2 normally. Exact shape/length/key/global
uniqueness validation remains. Matching EN/PL dialog/help/art.

## Catalog, ownership and verification

Punchbit1=2; ALL3145721->3145723,19->20items,524288->1048576 masks.
Append weight15 AFTERKick:395->410 interval[395,410),sole115. Canonical starts
kick,punch,wallPass,heart,... (generation append order differs). Preserve all old
maps/RNG/layers/pool boundaries; Bracelet2/Oil20 disabled.

Measure prior gate walltime/memory before batching/sharding million masks. Keep
EXACT full-message validation results for every mask in deterministic disjoint
ranges (e.g1024), fresh state/AJV reuse/rawmask failure, no hugecase array. Prove
full union; preserve invalid/archive/floor and ALL existing expensive old-mask
room/start/replay assertions. Default retains exhaustive replay extension; if
measured impractical, separately review proposed design before changing it.
No sampling/deleted checks/raised timeouts; shards need full range aggregate and
local unpartitioned full suite. Avoid crossproduct long interaction routes.

Ownership: domain/catalog/manual waves/private observations/tests first; then
v10 trusted facts/archive/dispatcher/mailbox/runtime/replay/shared-highwater;
then websender/bindings/preferences/UI/art; coordinator native/CI/version/docs/
measured tests. Serialize shared modules,400-line limit, independent domain/
queue/hazard and protocol/runtime/privacy/controls reviews.

Required: directions/fractional/heading/Reverse/wronggear/freeze/cooldown;
wrap/search/cycle/original/tangent/overlap/allbombs/actors; both claim priorities/
waves/Remote reservationchronology; sourcefire/due/pause/targetfire/oneplan;
captures/capacity/history/grace; mixed queues8/64/duplicate/fence/TTL/resume/padded
placement/Diarrhea; full rollback/replay; hidden invalid +second encode failure;
E-already-bound migration/key-held/remap/focus; same-root paint/quietaudio; masks/
legacy parity. Preserve16bombs195flames96events<64KiBframes<=64byte records<4MiB.

## Proposed native route, unexecuted

Prove ENTIRE seed31 solePunch ordinary-input route FIRST, including prep/upgrades/
fractional alignment/waits/escape with legal300 fuse. No assigned positions/items.
ENdesktop/PL320 ready. Reveal3,1 withbomb2,1; escape1,1->1,2/wait/collect.
Prep bomb3,1 clears3,2+4,1, escape2,1->1,1->1,2. Prep bomb4,1 clears5,1,
escape3,1->3,2, account collected base upgrades. Demo bomb4,1; fully exitEast5,1,
faceWest, actualE equipment action. SameID jumps4,1->1,1, unchangedcaptures,
no newplacement/motionaudio. Escape4,1->3,1->3,2 BEFOREscreenshots; prove natural
detonation at captureddeadline/bothalive. Cheapcompositorcrops destination and
unobscuredsourceabsence +ENPLgear/control text. Wrap/cooldown/mixed interactions
remain core/runtime. Actualkeys/UI/read-onlyobs, releasefinally; no injections/
seedsearch/retries/timeout weakening. Unchanged all/coverage/security/k8s gates,
bothprofiles/shuffledweb/independentreviews/fresh cumulativesameimageC+K/audio/
finalCI and corrected preflight/dephashes. FocusedPR45, paired deploy/rollback/
reload/newrooms; oldmask rejects, no DB migration.
