# Kick

Kick is opt-in permanent equipment in Expanded-v5 rooms. Walking voluntarily into a stationary
Timed or Remote bomb arms motion in the effective movement direction. The first one-cell hop occurs
on the next running update and later hops occur every ten updates until a hard wall, breakable block,
any bomb, or a living actor blocks the swept cells. Contact mines cannot be kicked.

Motion changes only the bomb's public cell. Its entity ID, owner, range, blast kind, placement tick
and captured fuse remain unchanged. A moved bomb remains fully solid in its current cell between
hops. Movement creates no placement, detonation or audio event. Timed bombs still detonate at their
captured deadline; Remote bombs retain their owner action trigger.

Direction, countdown and persistent `hasBeenKicked` provenance are private trusted state used for
capture, resume, replay and archive rejection. They never appear in gameplay-v11 wire state or
events. Public clients render the latest authoritative cell on the retained bomb drawing. Invisible
Bombs filtering and Invisibility Cloak attribution rules remain unchanged.

Version 0.3.44 added Kick as the first item in its nineteen-item catalog. Its capability bit
is `1`; `ALL` is `3,145,721`. Generator v2 appends weight 15 after Wall Pass, making the complete
pool 395 and Kick interval `[380,395)`. All 524,288 catalog subsets are validated. Archived gameplay
v9 and earlier reject Kick catalog entries, equipment, pickups, collection history, and trusted bomb
motion or stopped history. This document describes reviewed source; native same-image acceptance and
release publication remain pending.
