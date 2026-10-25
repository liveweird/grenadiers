# Wall Pass

Wall Pass is configurable permanent equipment in Expanded-v5 rooms and is enabled with every
available item in new rooms. It occupies the equipment slot independently of
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

## Native verification and published delivery

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

The pre-collection captures, refreshed on 2026-10-25 for the Napoleonic arena art, show the same
route in [English desktop](../screenshots/wall-pass-desktop.png) and
[Polish 320px reduced motion](../screenshots/wall-pass-mobile-pl.png). Their SHA-256 values are
`c2fcc6fbccd5ce8a5a8edc24b022b9ccfdaa762603e776812903bb9f0cee7ab8` and
`3c14790850e6ec21c4c417ee5e7df41a9f9207492ca0157a0cd470b09e25a5b9`.

Wall Pass is published as the eighteenth requested item in
[v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43). Published source
`c086f719ea470b9fe24963adef84b5e3bdf3e17a`, tree
`d74fad8a1c870f2834fca48f80ea4e38f0c512d9`, annotated tag
`c8395fd7dcc5701842644967802e61c76211931c`, and first-attempt candidate/master/tag workflows are
independently reviewed. Localhost serves the accepted image above from container
`d7f68cc89bfc87d06c202fbedae78c1892f54ea3ccdc6571fee0d1c5ea2f4240`.

The 110 native cases remain attributed to source `11d0dc7`. Corrected source `5dbcd58` updates the
two web dependency manifests to Happy DOM 21.0.4; reviewed artifact equality connects it to the
published source through eight documentation-only changes. This does not claim unchanged Docker
inputs, a new Rust compilation or new native execution. Kick is the v0.3.44 source-only candidate;
Punch, Bracelet and Oil Barrel remain before ART-ATLAS.
