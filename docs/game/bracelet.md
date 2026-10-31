# Bracelet

In live rules10, contact stops players before overlap. Walking into a rival's physical blocking
face can shove even when the holder's accepted movement is zero. Tile reservations without
physical contact do not trigger it; forced travel respects every player's starts and sweeps.

## Availability and rules

Bracelet is configurable permanent equipment enabled in new Expanded-v5 rooms. It has no manual action. When its
holder moves forward into the other living player, the authoritative movement step may shove that
player along the attempted direction. The victim moves by at most one tile; walls, soft blocks,
bombs, arena bounds and other collision rules clamp the destination. Swept fire and mine contact
still apply, and a shove never recursively shoves another actor.

A successful shove records private forced movement, starts a fixed 30-physical-tick (0.5 second)
cooldown and disarms further contact until the players separate. Both the cooldown and separation
conditions must clear before another shove can succeed. Failed, blocked, frozen, unequipped or
cooling-down contacts use ordinary movement results and do not start a cooldown. Simultaneous
claims use the round's alternating seat priority.

## State and privacy

The browser shows a collected Bracelet in public inventory views for visible players. English describes
“up to 1 tile” and a “0.5 second cooldown”; Polish describes “najwyżej o 1 pole” and “0,5 sekundy
odnowienia”. There is no Bracelet key, shove event, dedicated shove sound, countdown, destination hint or blocked
reason. The private cooldown marker and forced-movement report do not appear in recipient frames.

## Compatibility and verification

Bracelet uses equipment capability bit `4` and generator-v2 weight 15 after Punch. Archived gameplay v9
and earlier reject Bracelet settings, equipment, pickups, cooldown and forced-movement history.
See [English desktop](../screenshots/bracelet-desktop.png) and
[Polish 320px](../screenshots/bracelet-mobile-pl.png). The release record is in
[item acceptance history](../history/game-acceptance.md#bracelet-v0346).

Optional local movement audio may include short contact displacements indistinguishable from walking.
It provides no shove attribution or opponent-position cue; see [audio](audio.md).
