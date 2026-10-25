# Kick

## Availability and rules

Kick is configurable permanent equipment enabled in new Expanded-v5 rooms. Walking voluntarily into a stationary
Timed or Remote bomb arms motion in the effective movement direction. The first one-cell hop occurs
on the next running update and later hops occur every ten updates until a hard wall, breakable block,
any bomb, or a living actor blocks the swept cells. Contact mines cannot be kicked.

Motion changes only the bomb's public cell. Its entity ID, owner, range, blast kind, placement tick
and captured fuse remain unchanged. A moved bomb remains fully solid in its current cell between
hops. Movement creates no placement, detonation or audio event. Timed bombs still detonate at their
captured deadline; Remote bombs retain their owner action trigger.

## State and privacy

Direction, countdown and persistent `hasBeenKicked` provenance are private trusted state used for
capture, resume, replay and archive rejection. They never appear in gameplay-v11 wire state or
events. Public clients render the latest authoritative cell on the retained bomb drawing. Invisible
Bombs filtering and Invisibility Cloak attribution rules remain unchanged.

<a id="compatibility-and-verification"></a>

## Compatibility

Kick uses equipment capability bit `1` and generator-v2 weight 15. Archived gameplay v9 and
earlier reject Kick catalog entries, equipment, pickups, collection history, and trusted bomb
motion or stopped history.

## Examples

The Napoleonic arena captures show the stopped kicked bomb in
[English desktop](../screenshots/kick-desktop.png) and
[Polish 320px reduced motion](../screenshots/kick-mobile-pl.png). The release record is in
[item acceptance history](../history/game-acceptance.md#kick-v0344).
