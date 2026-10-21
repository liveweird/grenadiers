# Bracelet

Bracelet is opt-in permanent equipment in Expanded-v5 rooms. It has no manual action. When its
holder moves forward into the other living player, the authoritative movement step may shove that
player along the attempted direction. The victim moves by at most one tile; walls, soft blocks,
bombs, arena bounds and other collision rules clamp the destination. Swept fire and mine contact
still apply, and a shove never recursively shoves another actor.

A successful shove records private forced movement, starts a fixed 30-physical-tick (0.5 second)
cooldown and disarms further contact until the players separate. Both the cooldown and separation
conditions must clear before another shove can succeed. Failed, blocked, frozen, unequipped or
cooling-down contacts use ordinary movement results and do not start a cooldown. Simultaneous
claims use the round's alternating seat priority.

The browser shows the collected Bracelet in both players' public inventory views. English describes
“up to 1 tile” and a “0.5 second cooldown”; Polish describes “najwyżej o 1 pole” and “0,5 sekundy
odnowienia”. There is no Bracelet key, shove event, sound, countdown, destination hint or blocked
reason. The private cooldown marker and forced-movement report do not appear in recipient frames.

Version 0.3.46 adds Bracelet as the first item in the 21-item current catalog. Its capability bit is
`4`; `ALL` is `3,145,727`. Generator v2 appends weight 15 after Punch, making the complete pool 425
and the Bracelet interval `[410,425)`. A Bracelet-only pool is 115. All 2,097,152 catalog subsets
are validated, while all earlier masks preserve their v0.3.45 seeded output. Archived gameplay v9
and earlier reject Bracelet settings, equipment, pickups, cooldown and forced-movement history.
This document describes reviewed source; the full debug mask gate, native same-image acceptance,
accepted v0.3.45 parent integration, final CI and publication remain pending.
