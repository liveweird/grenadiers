# Power Bomb

Power Bomb is a configurable Expanded v5 pickup enabled in new rooms. It occupies the one-charge weapon slot and replaces
another stored weapon. A successful manual bomb placement consumes the charge and captures a
Power blast. Failed placements retain it. Placement precedes pickup collection, so an old charge
is used before a new weapon at the same cell is stored.

## Blast behavior

A Power blast burns every floor cell within the bomb's captured Manhattan range, including its
center. Hard walls remain intact and unburned, but do not stop fire from reaching eligible cells
beyond them. Every breakable block in the footprint burns and is destroyed; every bomb in the
footprint chains without shielding cells behind it. Chained bombs use their own captured blast
kind and range.

The complete explosion plan uses the immutable pre-explosion world. Ordinary and Piercing rays
therefore still stop at every original bomb cell, and ordinary rays still stop at original blocks
that a simultaneous Power blast destroys. Bomb IDs affect record order, not blast geometry.

The blast, range, fuse, owner, and placement tick are captured on placement. Later inventory
replacement, pickup expiry, death, or reconnect cannot change an existing bomb. Automatic
Diarrhea bombs are always ordinary and never inspect or consume the stored charge.

## Compatibility

Power Bomb uses catalog bit 7 and generator weight 15 after Piercing Bomb. Rules version 5, generator version 2 and journal format 2 remain
unchanged. Current [gameplay-v11](protocol-v11.md) requires paired server/web deployment and a
browser reload.

## Acceptance and presentation

The paired-browser scenario collects the pickup with ordinary key input, places an
armed Power bomb, verifies the Manhattan footprint through obstructing hard walls, and proves
consumption by placing a fresh Standard bomb afterward on both peers.

See the [English desktop](../screenshots/power-bomb-desktop.png) and
[Polish 320px](../screenshots/power-bomb-polish-320.png) examples. Native assertions establish
blast geometry, consumption and the following Standard bomb. The release record is in
[item acceptance history](../history/game-acceptance.md#power-bomb-v0335).
