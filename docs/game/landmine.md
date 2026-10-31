# Landmine

## Availability and placement

Landmine is a configurable single-use weapon, enabled in new rooms. A successful manual Bomb
action consumes the stored item and places a visible, passable Standard bomb with a Contact
trigger. It has no fuse.

In current rules10, a mine is additional to ordinary bomb capacity. You can plant it while all
regular slots are occupied, and it never blocks the next ordinary bomb. Detonating or destroying
it neither refunds a regular slot nor increases capacity. A later mine pickup can place another
mine while the first remains armed; each placement consumes its own item. One bomb occupies each
cell. Automatic Diarrhea placements remain ordinary Timed bombs and retain the mine item.
An existing Remote bomb is triggered before a new mine is placed.

## Contact and destruction

Actors initially overlapping a mine receive exit grace. Reconnect preserves existing grace
without granting more. Standing still or touching the boundary does not trigger contact. Grace
ends once the actor is fully outside; later swept entry, including the owner's, detonates it.
A mine cannot trigger itself during its placement tick.

Fire and other blasts destroy an untriggered mine without detonating it or extending flames;
blast rays continue through it. Contact already accepted in that tick takes precedence. Kick and
Punch do not move mines. Ordinary bomb and damage rules apply to the mine's explosion.

## Compatibility and accounting

Current [gameplay18](protocol-v18.md) reports `liveBombCount` as occupied ordinary slots, excluding
Contact mines. Visible mines remain in the bomb list; Invisible Bombs and cloak privacy still
apply. Rules1–8 retain historical capacity-counted mines. Arena generation and pickup weight 15
remain unchanged. Reload browsers after upgrading the paired server/web image.

Actual simulation tests cover placement at full capacity, repeated pickups, contact detonation,
Remote priority and archived accounting. The browser scenario covers collection, exit grace,
ordinary-slot counters, same-ID persistence and native contact detonation. Earlier release
acceptance is retained in [item history](../history/game-acceptance.md#landmine-v0337).
