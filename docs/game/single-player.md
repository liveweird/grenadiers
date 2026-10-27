# Solo match against the CPU

Choose **Play solo against CPU** in the lobby to create a private Standard match. You occupy
seat One and the server controls seat Two. Solo uses the same arena, movement, bombs, item rules,
round target and damage as a human duel. All available items start enabled.

## Settings, start and rematch

Configure the match before marking yourself Ready. The CPU acknowledges the current settings
automatically; your readiness starts the normal countdown. Settings changes clear readiness
and the CPU acknowledges the new revision. The interface labels the CPU and omits an invitation
code because another player cannot join this room.

Round results and the configured match target work normally. After the result hold, the CPU
consents to a rematch while you are connected. Choose Play again to return to Waiting with the
same rules, then Ready to start the next match.

## Reconnect and departure

A running transport interruption clears human input while physics and the CPU continue through
the existing reservation grace. Resume restores your human seat and preserves CPU memory and
randomness. A new physical round resets its round memory. Before the first start, a transport
loss releases the human immediately; it is not a paused or resumable solo room.

Explicit leave or grace expiry settles the ordinary outcome before closing the room. A newly
disconnected human after the final match result is released immediately. The CPU cannot retain
an empty room. Restarting the server ends solo and human-duel matches alike.

## Initial opponent behavior

The CPU scores reachable useful upgrades, Heart, beneficial buffs, block-opening positions and
visible rival pressure. It can take a detour rather than chase an unreachable nearer item, and
avoids deliberate travel through known curses or unsupported weapons. Capped upgrades and
harmless duplicates can still be crossed; held Heart is protected against equipment replacement. It
avoids known hazards and plants an ordinary bomb only when its restricted observation supports
a checked escape. Accepted movement and manual actions use the same validation as humans.
This is an initial opponent, with conservative tactics rather than tuned difficulty levels.

Around observed hazards, the CPU forecasts timed blasts, chains, flame expiry and persistent
Oil fire. It can plan several movement pulses and waits through a clear interval, including
Slow Foot and Reverse Controls expiry. A committed route continues without another planning
job, using short stops between moves and checking fresh observations against its schedule.
When complete route planning fails, a checked single-pulse fallback can still make progress;
that fallback does not guarantee completion of a later escape under worker delay.

When only finite flames remain, a committed route covers their expiry and its execution
margin, allowing a fresh decision afterward. Unlit Oil elsewhere does not suspend ordinary
goal movement; its own cell remains an obstacle. Active bombs and burning Oil retain the
longer conservative planning window.

For useful ordinary, Piercing and Power bombs, the CPU can check a complete escape against both
successful placement
and no placement. It accounts for movement before planting, range/fuse modifiers and effect expiry,
then confirms the actual visible bomb or the absent outcome. The same committed escape continues
without another worker. A rejected Bomb action is not retried. Conservative limits can still
refuse a useful bomb; the earlier restricted planting fallback remains available.

If another observed bomb interrupts a confirmed planting escape while the CPU still overlaps
its own origin, it stops the old route and checks fresh short escape movements. The original
placement record lasts only while its bomb, pose and player state remain valid; leaving the origin
ends that exception. Guidance keeps a stable legal exit target and requires progress toward it.
Recovery uses ordinary movement without planting another bomb.

Bomb motion remains partially unknown. Short conditional pulses may explicitly assume a bomb
has stayed stationary; observing it move withdraws that assumption. Complete conditional routes
retain the same explicit assumptions. Pauses and remote/contact
triggers cannot use this exception. Unknown bomb visibility, Diarrhea, unsupported collision
grace or an unavailable safe action produce Stop. New hazards can invalidate an active pulse.
The normal game still applies automatic curses and other effects.

Clear-board pickup collection and the restricted ordinary-bomb escape remain separate from
hazard pulses. Ordinary clear-board movement retains its accepted hold and can meet new hazards
before the next decision. Checked hazard movement avoids pickup-changing paths. Piercing and
Power charges use the same checked escape and consumption rules; Remote and Landmine are held.
This opponent does not predict new enemy actions or promise
survival against hidden facts.

Held Wall Pass lets the CPU route through known intact soft blocks. It keeps that equipment
rather than replacing it while relying on the passage. Known Oil remains conservatively avoided;
newly destroyed blocks stay fenced until their contents can be handled from fresh information.
Wall Pass never grants permission to plant inside a block or ignore fire, bombs or hard walls.

Ordinary navigation aligns within its current cell before attempting a turn outside the configured
turn tolerance. It keeps the chosen next cell, then decides the turn from a fresh stopped pose.
Alignment uses the normal movement speed and collision rules.

With Bracelet already held, the CPU can attempt a contact shove into visible existing fire.
It checks a straight approach and an open victim sweep, then certifies its own movement and
stopped suffix. This narrow tactic requires visible empty bombs, no exposed pickups and a
visible rival without Bracelet; changed facts withdraw the tactic. It uses ordinary movement
without a manual action. The rival may move away, and private cooldown or contact history may
prevent a shove. The CPU does not seek Bracelet specifically or relocate bombs for this tactic.

With Punch held, the CPU can check one stopped Equipment action against a single adjacent,
unpaused Standard timed bomb. It uses one complete escape for both relocation and cooldown
failure, then confirms the observed bomb identity and position. The narrow tactic requires a
centered CPU, no exposed pickups, Oil or existing flames, and a visible rival without Kick,
Punch or Bracelet. An ambiguous landing or failed proof withholds the action. This uses the
existing manual Punch rule and does not change player controls. New observations can invalidate
the route and require a fresh decision.

The [CPU contracts](cpu-contracts.md) explain execution and privacy; the
[accepted design](single-player-cpu.md) specifies the broader model. Remaining work and tuning
belong only in the [backlog](../../BACKLOG.md#single-player).
