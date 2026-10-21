# Power Bomb

Power Bomb is an opt-in Expanded v5 pickup. It occupies the one-charge weapon slot and replaces
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

Power Bomb uses catalog bit 7 and generator weight 15 after Piercing Bomb. Its initial catalog
supported all 2048 subsets and preserved every earlier 1024-mask path; later items join the
[current catalog](settings.md). Rules version 5, generator version 2 and journal format 2 remain
unchanged. Current [gameplay-v11](protocol-v11.md) requires paired server/web deployment and a
browser reload.
