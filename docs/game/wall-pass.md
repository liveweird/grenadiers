# Wall Pass

Wall Pass is opt-in permanent equipment in Expanded-v5 rooms. Heart remains the default preset;
room creators must enable Wall Pass explicitly. It occupies the equipment slot independently of
the single-use weapon slot, has no timer, and is replaced by another equipment pickup.

While held, a living actor may move through intact breakable blocks. Hard walls, arena bounds and
bombs remain solid. Contact mines, flames and explosions keep their ordinary collision and damage
rules: Wall Pass does not grant protection, destroy blocks, reveal concealed contents or collect an
item hidden under a block. Bomb placement still requires a legal center cell, so pressing the bomb
key while centered inside a breakable block creates no bomb and consumes no ID, capacity or weapon.

If another equipment pickup replaces Wall Pass while the actor overlaps one or more breakable
blocks, private per-cell exit grace lets the actor leave only those current overlaps. Grace ends for
each block when the complete actor footprint exits or merely touches its boundary; that block cannot
be re-entered. It has no timer, wire field, public marker or UI indicator. Heart consumption retains
existing exit grace. Death, reset and new-round construction clear it, while an ordinary reconnect
preserves authoritative state without granting new grace.

Teleporter validates each destination against the arriving actor's current Wall Pass capability and
own existing exit grace. A successful swap transfers no grace and prunes obsolete cells; a failed
swap remains atomic. Archived gameplay v9 and earlier catalogs reject Wall Pass state and history.
Gameplay v10 keeps the same rules version 5, generated-v2 arena and journal format 2.

Version 0.3.43 adds Wall Pass as the eighteenth current item. Its capability bit is `8`, so the
complete equipment mask becomes `3,145,720`. Generator v2 appends its weight-15 interval after
Teleporter: the all-items pool becomes 380, the interval is `[365,380)`, and the sole-item pool is
115. All 262,144 current catalog subsets are validated; every mask without Wall Pass retains its
earlier generated mapping and RNG draw count.

The deterministic browser source route uses the read-only repeating-byte-31 fixture. It reveals the
pickup at `(3,1)`, enters the intact block `(3,2)`, rejects placement there, clamps against hard wall
`(2,2)`, resumes inside through a newer admission, exits north and places a legal bomb with a
later entity ID, identical on both clients. E2E verifies no bomb, event or resource change for rejected
placement; focused core tests verify no entity allocation. That sole-item fixture cannot
exercise equipment replacement; focused core and protocol tests cover replacement exit grace.
