<a id="expanded-item-implementation-plan"></a>

# Expanded items

Expanded-v5 rooms expose all 22 requested item types, use generator v2, retain journal-v2 replay
compatibility and pair with gameplay v11. Older clients must reload. The incremental delivery record is in
[item acceptance history](../history/game-acceptance.md#expanded-item-delivery-sequence).

<a id="historical-heart-activation-checkpoint"></a>

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

Shield uses 300 physical ticks (five seconds); Stopwatch uses 180 (three seconds); Golden Flame, Short Fuse, Short Flame, Long Fuse, Reverse Controls, and Diarrhea use 600 (ten seconds), all
refreshing on recollection. Other timed effects and each Slow-foot stack use 600 ticks.
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
per seat/tick. Punch uses the remappable Equipment action, whose default is **E**.

| Item | Activation policy |
| --- | --- |
| Punch | Move the adjacent facing bomb three interior cells, wrapping playable coordinates. Landing requires floor, no soft block/barrel/other bomb, and no overlap with a living player's footprint. If blocked, try further three-cell landings for at most one axis cycle. Returning to the original cell fails unchanged, with no cooldown; successful punches have a 12-tick cooldown and stop sliding. |
| Bracelet | Moving contact shoves up to one tile, clamped by obstacles. Require separation to rearm and a 30-tick cooldown. Resolve simultaneous claims using alternating round priority; check swept fire/mines without recursive shoves. |
| Oil Barrel | At most eight generated arena objects reveal only when safe, block while unlit, and become passable shared fire until round end when ignited. |
| Kick | Push stationary Timed or Remote bombs one cell on the next update and every ten updates until blocked; Contact mines are excluded and identity, owner, blast and fuse remain unchanged. |
| Wall Pass | Ignore soft blocks, but not hard walls or solid bombs. Placement inside obstacles is illegal. Replacement grants private exit-only grace for positively overlapped soft cells; grace ends individually when the full footprint exits or reaches tangency. |
| Remote bomb | The next manual bomb action detonates the armed bomb instead of planting. Chains/fire still trigger it; death/disconnect adds no invented fuse. Another unused weapon may occupy the slot while it is armed. |
| Piercing bomb | Rays spend range through soft blocks/barrels; hard walls stop them. Solid bombs chain and stop the ray. |
| Power bomb | Affect eligible cells within Manhattan radius, ignoring line of sight. Hard walls remain intact without fire; blocks, barrels and bombs use the same immutable explosion plan. |
| Landmine | Passable, single-use and additional to ordinary slots in rules9; triggering or destroying it creates no slot refund. Initial overlapping actors have exit grace; later swept entry, including the owner, triggers it. Other blasts destroy it without detonation; an already accepted contact wins. Exclude mines from Kick/Punch. |

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

Gameplay-v11 defines checked inventory/effects, typed triggers, available items and an
explicit visible/hidden actor union. A hidden actor contains identity, seat
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

Teleporter is an immediate pickup effect. In the implemented two-seat rules it attempts one atomic
swap only when both actors are living and both complete destination footprints are within bounds
and clear of hard terrain, soft blocks, bombs and active fire. Contact overlap is safe only for the
arriving actor's already-valid grace; Shield does not make fire safe and grace never transfers.
Unsafe, dead or absent-target attempts consume the pickup without moving either actor. A living
disconnected reserved-seat actor remains eligible. Success preserves headings, clears residual and
turn intent, causes no swept contact or damage, and avoids destination pickup or extra action that
tick. It draws no RNG, occupies no slot and starts no timer. See [Teleporter](teleporter.md).

<a id="ordered-action-foundation"></a>
<a id="typed-domain-bomb-foundation"></a>
<a id="dormant-v7-contract"></a>
<a id="release-sequence-and-acceptance"></a>
