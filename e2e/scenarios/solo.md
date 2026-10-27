# Solo match against the server CPU

## Scenario: an Easy solo player completes, rematches, reconnects and leaves a real CPU match

One authenticated browser changes the default Standard selector to Easy and creates a private
solo room through the real interface.
The public room identifies seat one as human and seat two as a server-controlled CPU, exposes
no invitation code in the interface, and retains the default all-on item selection when the
creator changes the match target to one win and the fuse to 60 ticks. A separate authenticated
native socket cannot join the occupied room or resume the CPU seat with a fabricated capability.
The human alone readies the room, the real CPU moves, and the human plants a real bomb without
leaving its initial position. Either player may win because the arena and CPU decisions are
independently seeded. The authoritative winner must be alive, the loser eliminated, and the exact
one-win score and rendered status must name that winner.
One human rematch action gets the CPU's automatic consent, preserving the configured rules and
server-provided Easy difficulty label.
The human starts the rematch before reconnecting to its original running seat, then leaves;
the private room is removed. A transport loss before the first start is not resumable.

## Scenario: Polish solo controls default to Standard and stay keyboard accessible without horizontal overflow at 320 pixels

At a 320-pixel viewport the Polish lobby has no horizontal overflow, defaults to Standard,
allows keyboard difficulty selection, and starts a real Standard solo room.
The private-match difficulty,
Komputer player label, and Gotowy action remain visible while the invitation code stays hidden.
The waiting room has no horizontal overflow and passes the Axe accessibility audit.
