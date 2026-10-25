# Wall Pass

## Availability and rules

Wall Pass is configurable permanent equipment in Expanded-v5 rooms and is enabled with every
available item in new rooms. It occupies the equipment slot independently of
the single-use weapon slot, has no timer, and is replaced by another equipment pickup.

While held, a living actor may move through intact breakable blocks. Hard walls, arena bounds and
bombs remain solid. Contact mines, flames and explosions keep their ordinary collision and damage
rules: Wall Pass does not grant protection, destroy blocks, reveal concealed contents or collect an
item hidden under a block. Bomb placement still requires a legal center cell, so pressing the bomb
key while centered inside a breakable block creates no bomb and consumes no ID, capacity or weapon.

## State and privacy

If another equipment pickup replaces Wall Pass while the actor overlaps one or more breakable
blocks, private per-cell exit grace lets the actor leave only those current overlaps. Grace ends for
each block when the complete actor footprint exits or merely touches its boundary; that block cannot
be re-entered. It has no timer, wire field, public marker or UI indicator. Heart consumption retains
existing exit grace. Death, reset and new-round construction clear it, while an ordinary reconnect
preserves authoritative state without granting new grace.

## Compatibility

Teleporter validates each destination against the arriving actor's current Wall Pass capability and
own existing exit grace. A successful swap transfers no grace and prunes obsolete cells; a failed
swap remains atomic. Archived gameplay v9 and earlier catalogs reject Wall Pass state and history.
Gameplay v10 keeps the same rules version 5, generated-v2 arena and journal format 2.

Wall Pass uses equipment capability bit `8` and generator-v2 weight 15. Masks without Wall Pass
retain their generated mapping and RNG draw count.

The deterministic browser source route uses the read-only repeating-byte-31 fixture. It reveals the
pickup at `(3,1)`, enters the intact block `(3,2)`, rejects placement there, clamps against hard wall
`(2,2)`, resumes inside through a newer admission, exits north and places a legal bomb with a
later entity ID, identical on both clients. E2E verifies no bomb, event or resource change for rejected
placement; focused core tests verify no entity allocation. That sole-item fixture cannot
exercise equipment replacement; focused core and protocol tests cover replacement exit grace.

<a id="native-verification-and-published-delivery"></a>

## Examples

The Napoleonic arena pre-collection captures show the same
route in [English desktop](../screenshots/wall-pass-desktop.png) and
[Polish 320px reduced motion](../screenshots/wall-pass-mobile-pl.png). The release record is in
[item acceptance history](../history/game-acceptance.md#wall-pass-v0343).
