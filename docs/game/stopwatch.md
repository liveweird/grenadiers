# Stopwatch

## Availability and rules

Stopwatch is a configurable Expanded-v5 pickup, enabled in new rooms and active for 180 physical ticks. Recollection refreshes
the holder's deadline. While a living actor holds it, the opponent's voluntary movement and manual
actions are frozen. Network input is still admitted and fresh direction state replaces old state;
manual actions attempted while frozen are consumed rather than replayed after thaw. Two holders
freeze each other until their independent deadlines expire. Death clears the effect.

Every Timed bomb pauses once per physical update while any living Stopwatch holder exists. A bomb
placed during a pause keeps its complete captured fuse on the placement update. Remote and Contact
bombs remain untimed, and fire, chains, damage, effect timers, round deadlines, automatic Diarrhea
placement, publication and reconnect clocks continue. Each Timed bomb privately retains its
original captured fuse and accumulated paused ticks so live and historical wire facts cannot alias
an ordinary schedule. A newly placed bomb accumulates only the remaining active Stopwatch window,
from its placement through the holder's fixed deadline. All pause arithmetic is checked before the
update mutates state.

## State and privacy

Gameplay v9 publishes `voluntaryActionsFrozen` on visible actors and a pause flag on Timed bombs.
The flag is derived from both complete actors before Cloak recipient filtering; hidden actors stay
exactly identity, seat and alive status. Event audiences remain fixed when emitted. Gameplay v8 is
archived at the fourteen-item Cloak catalog and rejects Stopwatch, current pause and resumed pause
history even when a bomb is no longer paused.

## Compatibility

Version 0.3.40 adds Stopwatch as the fifteenth item and final generator weight 15. The all-items
pool grows from 320 to 335 entries. Every earlier 16,384-mask path is unchanged, while gameplay v9
supports all 32,768 subsets. Rules version 5, generator version 2 and journal format 2 are unchanged.

## Native acceptance checkpoint — 2026-10-21 20:56:41 UTC

Corrected source `fa25467b11038a9073d52f35e44ac6cee5380bcc`, tree
`07ac34a87a368d2c9425fcbc04020295ef9fe4e3`, and image
`sha256:ec8604af4317d188d7b00ab7cfa27115114430e0a11fb9821856617ad9409906`
passed complete 32,768-mask debug, release and coverage runs and fresh 52 Compose plus 52
Kubernetes cases. All 46 groups passed with zero retries and zero carried passes. The earlier
`bd9d064` attempt remains failed history: its fresh bomb was planted 34 ticks into the second
Stopwatch window, so the test oracle incorrectly expected 359 ticks instead of the correct 325;
Kubernetes never started. The correction derives remaining pause independently as
`deadline - placedAt - 1` and changes no product behavior, timeout, sample budget or retry policy.

Root reviewed two fresh Compose pre-collection captures: [English desktop](../screenshots/stopwatch-desktop.png)
and [Polish 320px reduced motion](../screenshots/stopwatch-mobile-pl.png). They show exposed clock
pickups and readable localized HUD/settings; they do not prove action freeze or bomb-pause behavior.
Those facts are established by the native assertions. Independent raw audit is clear and 2,434 payloads are durably sealed; hosted CI, PR/merge, publication, tag verification and promotion remained pending at this dated
checkpoint. Published/local v0.3.39 image `9a4dc108` remained the release baseline; see the
[release notes](https://github.com/liveweird/grenadiers/releases) for current status.
