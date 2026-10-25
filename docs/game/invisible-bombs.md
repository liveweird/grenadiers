# Invisible Bombs

<a id="release-status"></a>

## History

The release record is in
[item acceptance history](../history/game-acceptance.md#invisible-bombs-v0341).

## Availability and rules

Invisible Bombs is an optional ten-second curse in Expanded-v5 rooms. While it is active, the
affected player receives no live bomb records and no placement or detonation events. This applies
to their own and their opponent's Timed, Remote and Contact bombs, including paused Timed bombs and
special blast types. Bomb collision, capacity, contact, chains and detonation remain authoritative;
flames, terrain, pickups, results and each actor's live-bomb count stay public.

## State and privacy

The effect lasts exactly 600 physical ticks. Collection applies immediately through tick `T+599`,
expiry happens before actions at `T+600`, and recollection refreshes the deadline. Death and round
reset clear it. Resume preserves the original physical deadline and sends an event-free current
view. A surviving bomb appears at its current cell and remaining fuse after expiry or death; no
historical placement or sound is replayed.

Bomb event audiences are captured when the event occurs. Each event begins with the Cloak owner's
audience, then removes every seat blinded at that tick. An event may therefore be visible to both,
one seat, or neither. Earlier visible history remains visible after collection, and hidden history
does not become visible after expiry. Complete bomb and event facts are validated before either
recipient view is filtered and both recipient frames are prepared before publication advances.

## Compatibility

Invisible Bombs uses generator-v2 weight 15. Gameplay v9 remains archived at its sixteen-item
catalog, gameplay v8 remains pinned to its earlier Cloak-era catalog, and gameplay v10 and v11
preserve the same Invisible Bombs rules.

<a id="sealed-native-acceptance-checkpoint--2026-10-22-005523-utc"></a>

## Examples

The [English desktop](../screenshots/invisible-bombs-desktop.png) and
[Polish 320px reduced-motion](../screenshots/invisible-bombs-mobile-pl.png) pre-collection captures
show the exposed pickup and readable localized presentation. Privacy, reconnect and exact
expiry are checked by paired runtime assertions.
