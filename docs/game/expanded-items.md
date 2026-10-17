# Expanded item implementation plan

Status: **inventory/effects v0.3.19 and ordered actions v0.3.20 implemented; no items activated**. Classic remains
the default. This optional track follows the [backlog](backlog.md), shared settings and
pixel-art increments. The historical catalog in [rules](rules.md) is not an implementation
claim. Each activation requires its own reviewed behavior, UI, documentation and acceptance.

The foundation implements `core::game::{inventory,effects,damage}` and immutable Actor inventory,
effect and derived-stat access. Production cannot inject items into State yet. Both fire checks
share the damage gateway; effects/grace expire before movement and new bombs capture derived
range/fuse. Slow-foot exposes an exact speed ratio but does not change movement before activation.
No item pickup, equipment action, visible effect, new rules profile or wire field is enabled.

Independent review is clear after replacing overflow-prone Heart expiry with checked elapsed
grace. The integrated gates pass 334 core tests, 163 server tests, two doctests and 230 web tests;
four dedicated performance profiles remain intentionally ignored. Rust line coverage is 94.21%
above the unchanged 80% floor. Dependency audits and strict Kubernetes schemas pass. The same
image passes 24/24 scenarios through Compose and 24/24 through Kubernetes, retries zero,
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

Proposed durations at 60 Hz: Shield 300 ticks; Stopwatch 180; other timed effects and each
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

## Release sequence and acceptance

1. Pure core inventory/effects/damage foundation; retain classic traces and protocol behavior.
2. Ordered manual commands with a Classic simultaneous-placement adapter (v0.3.20).
3. Typed bomb triggers and a checked legacy projection boundary, then forced-contact helpers.
4. Gameplay-v7 schema/projections, followed by per-seat transport and browser inventory/status UI.
5. Expanded configuration/generation, then Heart and Shield as separate activations.
6. Individual stat modifiers, Reverse/Diarrhea, special bombs, movement equipment, Stopwatch,
   visibility effects, Oil and Teleporter, with interaction tests at each activation.

Each release needs a version bump, focused PR, independent review and existing gates. Test
exact expiry, protection priority, modifiers together, independently expiring stacks, action
ordering/reconnect, immutable chains, forced hazards, visibility during report windows and
resumed/terminal frames. Preserve historical replay parity and deterministic generation.
Verify entity/frame/journal bounds and expanded performance. Activated items require native
browser scenarios through Compose and Kubernetes using the same image, plus EN/PL feedback
and pixel-art indicators. Foundations alone do not claim playable item support.
