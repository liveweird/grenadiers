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

Final source `11d0dc7e39e8e2cafcf70c1548be339f52c740ab`, tree
`655c7d8a80b4dd3fdd3b232534c2cbaa45b054cc`, is native verified on exact image
`sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`.
At the `2026-10-24 01:00 UTC` checkpoint, all 55 Compose plus 55 Kubernetes cases pass with zero
retries or carried results. The matrix retains 52 reports, 122 PNGs, 42 runtime records, 21
Kubernetes identities, 20 fixtures, 40 seed records and 80 latency samples below 250 ms. Fresh
source gates pass all 262,144 masks in each of debug, release and coverage, 711 core and 209 server
tests per profile with four historical ignores, 797 web tests across 64 files and all 56 scenarios.
Independent audit, result review and binding review are clear. Final manifest
`3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c` durably seals 2,665
payloads; seal `31fe5eef2900a0229d002e45b5a310888f3915829db0aaf23bc3535a8eea5e2c`
is complete, and independent post-seal review is clear.

The reviewed pre-collection captures show the same route in [English desktop](../screenshots/wall-pass-desktop.png)
and [Polish 320px reduced motion](../screenshots/wall-pass-mobile-pl.png). Their SHA-256 values are
`d46367b7eeee7c55e94dbcfba6de7de50b27b96c6761ad6a9ff69db2b9332c08` and
`84843190d2be59a89f5c614c3d17bd78e969dcb7cf692d0e06243b2d3978d72f`.

This is the eighteenth native-verified requested item, not a v0.3.43 publication claim. Candidate,
master and tag checks, publication and exact-image promotion remain pending; consult current
[checks](https://github.com/liveweird/grenadiers/actions) and
[releases](https://github.com/liveweird/grenadiers/releases). Kick, Punch, Bracelet and Oil Barrel
remain before ART-ATLAS.
The later `5dbcd58` release candidate updates only the two web dependency manifests to Happy DOM
21.0.4. Its reviewed host distribution remains byte-equal to this accepted image, but that does
not relabel these 110 native cases; exact-head hosted and delivery checks remain required.
