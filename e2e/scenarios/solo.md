# Solo match against the server CPU

## Scenario: a solo player completes, rematches, reconnects and leaves a real CPU match

One authenticated browser creates a private Standard solo room through the real interface.
The public room identifies seat one as human and seat two as a server-controlled CPU, exposes
no invitation code in the interface, and retains the default all-on item selection when the
creator changes the match target to one win and the fuse to 60 ticks. A separate authenticated
native socket cannot join the occupied room or resume the CPU seat with a fabricated capability.
The human alone readies the room, the real CPU moves, and the human plants a real bomb without
leaving its initial position. The explosion eliminates the human and awards the CPU the match.
One human rematch action gets the CPU's automatic consent, preserving the configured rules.
The human starts the rematch before reconnecting to its original running seat, then leaves;
the private room is removed. A transport loss before the first start is not resumable.

## Scenario: Polish solo controls stay accessible without horizontal overflow at 320 pixels

At a 320-pixel viewport the Polish lobby starts a real solo room. The private-match difficulty,
Komputer player label, and Gotowy action remain visible while the invitation code stays hidden.
The waiting room has no horizontal overflow and passes the Axe accessibility audit.
