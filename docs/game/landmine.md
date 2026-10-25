# Landmine

## Current verification

Remote and Landmine use fresh placement-ID checks across rolling observer history. The release
record is in [item acceptance history](../history/game-acceptance.md#landmine-v0337).

## Availability and rules

Landmine is a configurable Expanded-v5 one-charge weapon enabled in new rooms. A successful manual bomb action consumes
the stored charge and places a visible, passable, capacity-counted Standard bomb with a Contact
trigger. It has no fuse. Automatic Diarrhea placements remain ordinary Timed bombs and do not
consume the charge.

## State and contact

Every actor overlapping a mine during initial State construction receives exit grace, and newly
placed mines grant grace to actors overlapping their cell. Reconnect and resume preserve the
existing simulation grace state; they do not grant fresh grace.
Standing still and boundary tangency do not trigger contact. Grace ends only after the actor is
fully outside; any later swept entry, including the owner's, detonates the mine. Only mines that
existed before movement are eligible, so placement cannot trigger itself in the same tick.
Contact accepted during movement remains authoritative if the actor subsequently dies in existing
fire during that tick.

Fire and other bomb blasts destroy an untriggered mine without detonating it, publishing a
`BombDetonated` event, or extending flames. Blast rays continue through the destroyed mine. An
accepted contact wins over destruction in the same immutable explosion plan. Timed and Remote
bombs retain their original obstruction snapshot and chain behavior.

## Compatibility

Landmine uses generator weight 15 after Remote Bomb.
Rules version 5, generator version 2 and journal format 2 remain unchanged. Current deployments
use [gameplay-v11](protocol-v11.md); reload older browsers with the paired server/web upgrade.

<a id="historical-v0337-native-acceptance"></a>

## Verification and examples



Both clients observe the single consumed charge, Contact/Standard bomb, persistence beyond
183 ticks, exact-ID contact detonation with a live origin flame, no extra placement, owner
elimination and seat Two's round win. The following fresh Timed Standard bomb is covered by
`consumed_landmine_is_followed_by_a_fresh_standard_timed_bomb_in_the_same_state` in the
[core tests](../../core/src/game/simulation/landmine_tests.rs), rather than a separate browser
assertion.

The [English desktop](../screenshots/landmine-desktop.png) and
[Polish 320px reduced-motion](../screenshots/landmine-polish-320.png) captures show the planted
pressure plate before detonation and normal HUD layout.
