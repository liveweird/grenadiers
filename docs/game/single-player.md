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

The CPU pursues exposed pickups, useful block-opening positions and visible rival pressure. It
avoids known hazards and plants an ordinary bomb only when its restricted observation supports
a short escape route. Accepted movement and manual actions use the same validation as humans.
This is an initial opponent, with conservative tactics rather than tuned difficulty levels.

Unknown bomb visibility, Diarrhea, Slow Foot and Reverse Controls stop its voluntary movement
until the unsupported condition ends. The normal game still applies automatic curses and other
effects. It does not predict general chain reactions, moving bombs or an opponent's future
actions. A proved route cannot protect it from newly introduced hazards or control effects.
Stored special weapons are held rather than used tactically by this initial opponent.

The [CPU contracts](cpu-contracts.md) explain execution and privacy; the
[accepted design](single-player-cpu.md) specifies the broader model. Remaining work and tuning
belong only in the [backlog](../../BACKLOG.md#single-player).
