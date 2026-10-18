# Invisible Bombs

Invisible Bombs is an optional ten-second curse in Expanded-v5 rooms. While it is active, the
affected player receives no live bomb records and no placement or detonation events. This applies
to their own and their opponent's Timed, Remote and Contact bombs, including paused Timed bombs and
special blast types. Bomb collision, capacity, contact, chains and detonation remain authoritative;
flames, terrain, pickups, results and each actor's live-bomb count stay public.

The effect lasts exactly 600 physical ticks. Collection applies immediately through tick `T+599`,
expiry happens before actions at `T+600`, and recollection refreshes the deadline. Death and round
reset clear it. Resume preserves the original physical deadline and sends an event-free current
view. A surviving bomb appears at its current cell and remaining fuse after expiry or death; no
historical placement or sound is replayed.

Bomb event audiences are captured when the event occurs. Each event begins with the Cloak owner's
audience, then removes every seat blinded at that tick. An event may therefore be visible to both,
one seat, or neither. Earlier visible history remains visible after collection, and hidden history
does not become visible after expiry. Complete bomb and event facts are validated before either
recipient view is filtered and both recipient frames are prepared before publication advances.

Version 0.3.41 adds Invisible Bombs as gameplay v9's sixteenth item and generator-v2's final weight
15. The all-items mask is `1048560`; all 65,536 subsets are accepted. The new interval is `[335,
350)` and the sole-item pool is 115. Every earlier catalog mask and generator path is unchanged.
Archived gameplay v8 remains pinned to its fourteen-item Cloak-era catalog.
