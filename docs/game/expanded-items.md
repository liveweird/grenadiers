# Expanded item implementation plan

Status: [Heart](heart.md), [Shield](shield.md), [Golden Flame](golden-flame.md),
[Short Fuse](short-fuse.md) and [Short Flame](short-flame.md) are published through v0.3.29.
The v0.3.30 [Long Fuse](long-fuse.md) increment passes local checks and corrected-source native
acceptance: 32/32 Compose and 32/32 Kubernetes cases, wrapper exit zero, no retries or carried
results. Independent final evidence review and all required CI pass; v0.3.30 is published. Verification
retains the earlier hosted failure and separately named source-specific attempts.
The v0.3.31 [Reverse Controls](reverse-controls.md) increment passes local checks and fresh
33/33 Compose plus 33/33 Kubernetes acceptance. All required CI passes; v0.3.31 is published.
The v0.3.32 [Diarrhea](diarrhea.md) increment passes local checks and independent implementation
reviews and fresh same-image 45/45 Compose plus 45/45 Kubernetes acceptance after the reviewed
startup-readiness correction. Independent final native review, candidate CI and post-merge CI
pass; [v0.3.32](https://github.com/liveweird/grenadiers/releases/tag/v0.3.32) is published. The
original CI counterexample is retained separately. The
v0.3.33 [Slow foot](slow-foot.md) passes local checks and independent implementation reviews and
fresh same-image 46/46 Compose plus 46/46 Kubernetes acceptance after reviewed browser-driver
corrections. All 22 latency cases per target pass; static parity contains 47 scenarios. The whole
wrapper exits zero with no retries or carried passes. Candidate and post-merge CI pass, and
[v0.3.33](https://github.com/liveweird/grenadiers/releases/tag/v0.3.33) is published as a source
prerelease. Its final raw source is integrated into the v0.3.34
[Piercing Bomb](piercing-bomb.md) candidate. Piercing retains its feature and version; repository,
E2E lint, Knip, type checking and 48-scenario parity pass. Fresh same-image acceptance passes all
47 Compose and 47 Kubernetes cases with zero retries. Independent evidence review, candidate CI
and post-merge CI pass; [v0.3.34](https://github.com/liveweird/grenadiers/releases/tag/v0.3.34)
is published as a source prerelease. Its final raw source is integrated into the v0.3.35
[Power Bomb](power-bomb.md) candidate. Power retains its feature and version; local checks and
independent implementation reviews are clear. Fresh same-image acceptance passes all 48 Compose
and 48 Kubernetes cases with zero retries; independent evidence review and the final seal are
clear; both five-job candidate CI runs and the five-job post-merge run pass, PR 81 is merged, and
[v0.3.35](https://github.com/liveweird/grenadiers/releases/tag/v0.3.35) is published as a source
prerelease and its accepted image is deployed locally. The accepted raw source is integrated into the
v0.3.36 [Remote Bomb](remote-bomb.md) candidate. Remote Bomb retains its reviewed one-charge
manual-trigger behavior and passes fresh same-image 49/49 Compose plus 49/49 Kubernetes cases
with zero retries. The raw evidence audit is clear and sealed; native evidence and documentation review are clear; publication requires exact-head candidate
and post-merge CI. Its CI-corrected accepted raw source is integrated into the
v0.3.37 [Landmine](landmine.md) source candidate. Landmine retains reviewed contact, per-actor
exit grace, silent destruction and immutable-planning behavior; fresh native acceptance and final
CI remain pending. Expanded-v5/generator-v2 rooms and gameplay-v8 are retained; see
[verification](../verification.md) and
[release notes](https://github.com/liveweird/grenadiers/releases).
This optional track follows the [backlog](backlog.md), shared settings and
pixel-art increments. The historical catalog in [rules](rules.md) is not an implementation
claim. Each activation requires its own reviewed behavior, UI, documentation and acceptance.

The v0.3.19 foundation introduced `core::game::{inventory,effects,damage}` and immutable Actor
inventory, effect and derived-stat access. At that stage, production could not inject items into
State. Both fire checks share the damage gateway; effects/grace expire before movement and new bombs capture derived
range/fuse. Slow-foot exposes an exact speed ratio but does not change movement before activation.
That foundational release enabled no item pickup, equipment action, visible effect, new rules
profile or wire field. Heart activation followed in v0.3.25.

The historical foundation review cleared checked elapsed grace in place of overflow-prone Heart
expiry. Its integrated gates passed 334 core tests, 163 server tests, two doctests and 230 web tests;
four dedicated performance profiles were intentionally ignored. Rust line coverage was 94.21%
above the unchanged 80% floor. Dependency audits and strict Kubernetes schemas passed. The same
image passed 24/24 scenarios through Compose and 24/24 through Kubernetes, retries zero,
including a recheck of the corrected default browser harness. See [verification](../verification.md)
for image identity, the final evidence rollup and its limits.

## State and timing

Equipment and single-use weapons occupy independent, replaceable slots. Both, together
with effects and damage grace, clear on death and round reset. Base speed, range and capacity
upgrades remain separate from temporary overlays. A weapon is consumed only by successful
manual placement; failed attempts do not discard it.

Effects expire before movement and damage at their exact expiry tick. Re-picking an ordinary
effect refreshes its expiry, without adding strength or duration. Slow-foot has at most six
independent stacks; every accepted stack halves current effective speed. At capacity, that
pickup is ineligible and remains available to the opponent, rather than being consumed without
another halving. Use fixed-point calculations without accumulating rounding errors.

Damage passes share one gateway. Shield prevents damage before Heart is considered. Heart
absorbs one lethal hit, consumes its equipment slot and grants 60 ticks of immunity, including
both damage passes and overlapping flames. Collecting protection cannot undo earlier damage.

Shield uses 300 physical ticks (five seconds); Golden Flame, Short Fuse, Short Flame, Long Fuse,
Reverse Controls and Diarrhea use 600 (ten seconds), all
refreshing on recollection. Proposed remaining durations: Stopwatch 180; other timed effects and each
Slow-foot stack 600. These become versioned constants when activated, not new settings yet.
Golden flame gives newly placed bombs radius eight; Short-flame overrides it with radius one.
Short fuse halves and Long fuse doubles the configured fuse; together they cancel. Derive
the result once, with checked arithmetic. Remote and contact triggers have no timed fuse.

## Bombs, commands and forced movement

Use explicit timed, remote and contact triggers; never fake a distant detonation deadline.
Keep immutable-world explosion/chain planning. Retain captured ownership, range and weapon
properties when a bomb moves. Separate bounded forced movement/contact traces from voluntary
turn traces. Teleport validates destinations without treating the intervening space as travel.

Manual bomb/equipment actions need a bounded ordered queue within the existing eight-command
per-seat/tick limit. Consume their shared action ID on admission, including failed gameplay
attempts; restore the high-water in reconnect baselines. Permit at most one successful placement
per seat/tick. Add a remappable equipment action, proposed default **E**, for Punch.

| Item | Activation policy |
| --- | --- |
| Kick | Walking into a solid bomb starts sliding next tick, one cell per ten ticks. Swept collisions stop at the last legal cell before walls, blocks, barrels, bombs or living players. |
| Punch | Move the adjacent facing bomb three interior cells, wrapping playable coordinates. Landing requires floor, no soft block/barrel/other bomb, and no overlap with a living player's footprint. If blocked, try further three-cell landings for at most one axis cycle. Returning to the original cell fails unchanged, with no cooldown; successful punches have a 12-tick cooldown and stop sliding. |
| Bracelet | New moving contact shoves up to one tile, clamped by obstacles. Require separation to rearm and a 30-tick cooldown. Resolve simultaneous claims using alternating round priority; check swept fire/mines without recursive shoves. |
| Wall pass | Ignore soft blocks and unlit barrels, but not hard walls or solid bombs. Placement inside obstacles is illegal. Replacement grants exit-only grace for obstacles currently overlapped; grace ends individually on exit. |
| Remote bomb | The next manual bomb action detonates the armed bomb instead of planting. Chains/fire still trigger it; death/disconnect adds no invented fuse. Another unused weapon may occupy the slot while it is armed. |
| Piercing bomb | Rays spend range through soft blocks/barrels; hard walls stop them. Solid bombs chain and stop the ray. |
| Power bomb | Affect eligible cells within Manhattan radius, ignoring line of sight. Hard walls remain intact without fire; blocks, barrels and bombs use the same immutable explosion plan. |
| Landmine | Passable and capacity-counted. Initial overlapping actors have exit grace; later swept entry, including the owner, triggers it. Other blasts destroy it without detonation; an already accepted contact wins. Exclude mines from Kick/Punch. |

Reverse controls transforms held directions within the simulator while retaining raw input
press/release identity; clear pending turn assistance when inversion changes. Diarrhea makes
at most one ordinary placement attempt per tick, after manual priority, using available capacity.
It never consumes a weapon or activates a remote.

Stopwatch pauses timed fuses and opponents' voluntary actions/movement. Fire, bomb motion,
chain/contact/remote triggers, effect timers, round deadlines and network clocks continue.
Fresh movement input still replaces held state; frozen actors accumulate no delayed actions.
Two holders freeze each other until independently expiring effects release them. Existing timed
deadlines advance once per paused update; newly placed bombs retain exactly their initial
remaining fuse without an extra planting-tick increment. Preflight deadline overflow atomically.

## Recipient visibility and contracts

Gameplay-v7 must define checked inventory/effects, typed triggers, available items and an
explicit visible/hidden actor union before activation. A hidden actor contains identity, seat
and alive status only; its owner always receives spatial state. Timed triggers expose remaining
ticks and pause state. Visible bomb lists cannot stand in for authoritative capacity counts.

Validate the canonical world before filtering. Build and validate two bounded seat-specific
encodings atomically; route by authenticated seat during publication, admission, resume,
replacement, retirement and shutdown. Clear reports once after both perspectives are built.
Capture visibility when events occur, including effects expiring within a publication window;
filter before encoding and assign recipient-local contiguous event indices.

Cloak suppresses opponent actor coordinates and attributable spatial events/audio. Invisible
bombs suppresses every bomb record and placement/movement cue for that recipient, including
their own. Resulting fire remains visible. Remove hidden objects from interpolation, trails,
shadows, accessibility/debug and audio; reveal by snapping, without interpolating from stale
hidden positions. Public arena changes and visible bombs remain inference clues; packet
filtering does not promise absolute untrackability.

Expanded configuration uses a compact enabled-item set to preserve the 64-byte replay record
and journal below 4 MiB. Advertise available items and reject enabling unfinished ones. Freeze
each activated rules/generator identity; Classic retains rules-v4/generator-v1 behavior.
Do not truncate events to the old limit: derive bounds and prove both maximum encoded frames
remain below 64 KiB. Record the complete chosen configuration privately without disclosing seeds.

## Oil and teleport interpretations

Oil barrel is an arena object, rather than an inventory timer: a safely revealed oil drop
becomes solid, with at most eight per arena. Ignition leaves fire on that cell until round end,
without extra rays. Ordinary rays stop there; piercing/power retain their specified behavior.
Document this interpretation visibly when enabling it, since the request groups it with curses.

Teleporter is an immediate pickup effect. Swap with an eligible living rival only when both
destination footprints are legal and free of active fire or triggerable mines; otherwise
consume without swapping. Preserve heading, clear movement residual/turn intent and avoid
another destination pickup that tick. Simultaneous pickups cause one priority-resolved swap.
With two humans there is only one possible rival; random target selection waits for more seats.

## Ordered-action foundation

The v0.3.20 foundation retains individual accepted manual actions through server admission and
core input processing. Classic still resolves one simultaneous placement claim per seat;
sequential retries must not let duplicate requests defeat a shared-cell conflict. Raw-core
Clear cancels earlier actions; server movement clearing and TTL expiry preserve accepted
manual actions because movement is drained first. Fences discard queued actions but retain
the admitted action high-water. New round construction resets it.

The existing eight-command allowance applies to a pending mailbox batch and resets on drain
or fence. A stricter cumulative budget across same-tick replacements needs a separate,
versioned private admission policy. Preserve the core's separate 64-logical-input limit.
Typed untimed bombs and new equipment controls are not part of this preservation increment.
Replay checkpoints compare the complete pending queue; journal records and public v6 stay unchanged.

## Typed domain bomb foundation

The v0.3.21 foundation distinguishes Timed, Remote and Contact triggers in core bomb records
and domain views/events. Untimed records have no deadline. Current State construction,
stepping and Classic explosion planning reject unsupported triggers before simulation mutation;
production placement still creates only Timed bombs with checked fuse arithmetic.

Legacy wire bomb records retain mandatory `detonatesAt`. Their shared checked projection
rejects Remote/Contact in both snapshots and event windows, including events for bombs absent
from final state. One unsupported record fails the whole window; no timestamp is invented and
no event is dropped. Failed publication preserves revision and pending reports. These changes
do not activate remote control, mine contact/destruction, fuse pausing or bomb movement.

## Dormant v7 contract

The v0.3.22 increment stages schema, generated browser types, Rust wire DTOs and checked-value
helpers only. The then-active gameplay-v6 and its archived contracts remain unchanged. The contract
includes inventory, 11 timed effect kinds (at most 16 entries), explicit bomb triggers, a
visible/hidden actor union, available/enabled items and shared manual action IDs.

JSON Schema/AJV checks server shapes, required nulls, ranges and array bounds. Rust parsing
checks incoming client messages, settings and checked values; server DTOs are serialization-only.
Complete server-state validation belongs to the later canonical projection increment.
Separate matching semantic helpers check ordinary-effect uniqueness, the six-stack Slow-foot
cap and enabled-item availability. Equal Slow-foot durations remain valid. Raw AJV validation
alone does not establish these semantic properties. Checked item sets use 22 fixed bits and
canonical wire order, without enlarging current live room settings or journal records.

Canonical complete-world validation, event-time audience capture, recipient projections and
live transport/browser migration are separate increments. See [v7 contract](protocol-v7.md).

## Release sequence and acceptance

1. Pure core inventory/effects/damage foundation; retain classic traces and protocol behavior.
2. Ordered manual commands with a Classic simultaneous-placement adapter (v0.3.20).
3. Typed domain triggers and checked legacy wire projection (v0.3.21). Forced-contact
   helpers follow playable protection items rather than delaying them.
4. Dormant gameplay-v7 schema/DTOs/checked values (v0.3.22 candidate).
5. Canonical validation and recipient projections (v0.3.23 candidate), with strict Classic
   binding and event-time audience capture; no live endpoint or item activation.
6. Integrated server/browser v7 migration (v0.3.24 candidate), including recipient-specific
   admission and routing; Classic still has empty item availability.
7. Expanded configuration/generation with Heart activation, then Shield in a separate increment.
8. Individual stat modifiers, Reverse/Diarrhea, special bombs, movement equipment, Stopwatch,
   visibility effects, Oil and Teleporter, with interaction tests at each activation.

Each release needs a version bump, focused PR, independent review and existing gates. Test
exact expiry, protection priority, modifiers together, independently expiring stacks, action
ordering/reconnect, immutable chains, forced hazards, visibility during report windows and
resumed/terminal frames. Preserve historical replay parity and deterministic generation.
Verify entity/frame/journal bounds and expanded performance. Activated items require native
browser scenarios through Compose and Kubernetes using the same image, plus EN/PL feedback
and pixel-art indicators. Foundations alone do not claim playable item support.
