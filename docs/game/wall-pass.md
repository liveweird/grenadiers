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

## Native verification checkpoint

Final source `bd34192d8915a8aac4288e3f69bd5898ad01d952`, tree
`daa2306e77de0e279a6752aa69e687e0a6c51ba0`, is native verified on exact image
`sha256:e2f02586988e2359404189afbe465ad47a4dd12e8227de8b6d3c627401ad7cfc`.
At the `2026-10-23 19:48 UTC` checkpoint, all 55 Compose plus 55 Kubernetes cases pass with zero
retries or carried results. The matrix retains 52 reports, 122 PNGs, 42 runtime records, 21
Kubernetes identities, 20 fixtures, 40 seed records and 80 latency samples below 250 ms. Fresh
source gates pass all 262,144 masks in each of debug, release and coverage, 711 core and 209 server
tests per profile with four historical ignores, 797 web tests across 64 files and all 56 scenarios.
Independent audit and result/binding reviews are clear, and final manifest
`e2f58e9a171b62eb2607d46a59236f19e2ca99fac0a7cd1148d3a34d5fae1a0e` durably seals the evidence.

The reviewed pre-collection captures show the same route in [English desktop](../screenshots/wall-pass-desktop.png)
and [Polish 320px reduced motion](../screenshots/wall-pass-mobile-pl.png). Their SHA-256 values are
`9200da355aa5fe9273a9a130186c7d8f27d28255f705abd6ded068671e71f9f8` and
`07a276234fdf8023be1603fd04c27965ef4fa8698afbef63623c84205a0bf4ec`.

This is the eighteenth native-verified requested item, not a v0.3.43 publication claim. Candidate,
master and tag checks, publication and exact-image promotion remain pending; consult current
[checks](https://github.com/liveweird/grenadiers/actions) and
[releases](https://github.com/liveweird/grenadiers/releases). Kick, Punch, Bracelet and Oil Barrel
remain before ART-ATLAS.
