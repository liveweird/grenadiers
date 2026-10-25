# Invisibility Cloak

## Availability and rules

Invisibility Cloak is a configurable Expanded-v5 pickup enabled in new rooms that lasts 600 physical ticks. Recollection
refreshes that deadline. It coexists with equipment, weapons and other effects, changes no movement,
damage or bomb rule, and clears on death and round reset.

## State and privacy

The owner receives their complete actor. An opponent receives exactly the actor's ID, seat and alive
status while the effect is active; position, heading, inventory, effects, grace and derived stats are
absent. Bombs, flames, blocks, pickups and results remain public, so visible world changes can still
reveal clues. Expiry reveals the actor at their current position without interpolating from a stale
location. Disconnect and resume preserve the physical deadline and do not replay private history.

Pickup collection, bomb placement and detonation, and damage-prevention events are private to an
affected cloaked seat when each event occurs. Elimination and round-result events remain public.
Audience is captured at emission and is not recomputed from the final world in a publication window.
Both complete recipient encodings are validated before either is published; legacy protocols reject
the private semantics.

## Compatibility

Invisibility Cloak uses generator weight 15 after Landmine. Its fourteen-item catalog remains
pinned by archived gameplay v8; the live contract is [gameplay v11](protocol-v11.md).
Rules version 5, generator version 2 and journal format 2 remain unchanged. Reload older browsers
when deploying the paired server/web upgrade.

## Verification

The [English desktop](../screenshots/invisibility-cloak-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisibility-cloak-polish-320.png) captures show the
Cloak pickup before collection and readable localized HUD/settings. These stills do not prove
actor privacy: the paired native assertions establish owner
visibility, exact peer-hidden actor shape, compositor removal, reconnect and current-position
reveal at the physical deadline.

The release record is in
[item acceptance history](../history/game-acceptance.md#invisibility-cloak-v0338-v0340).
