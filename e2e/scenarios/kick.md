# Kick

## Scenario: Kick moves one retained bomb through authoritative cells without extra cues and preserves its fuse

Two Chromium contexts use the repeating-byte-31 generated-v2 fixture with only Kick enabled and a
300-tick fuse. The English desktop and Polish 320-pixel reduced-motion clients ready before play,
reveal and collect Kick with an ordinary bomb, then use a second ordinary bomb to clear the escape
corridor. A demo bomb is planted at `(3,1)` and the holder walks west into it using real keyboard
input. Both public streams retain the same entity ID and observe ordered cells `(3,1)`, `(2,1)`,
`(1,1)` with unchanged owner, range, placement tick, Timed trigger and Standard blast. The movement
creates no extra placement event. Both clients capture the stopped bomb and vacated source cell.
The holder escapes through `(3,2)` and both clients observe natural detonation at `placedAt + 300`
from `(1,1)` while both actors survive. The scenario uses ordinary UI actions, native keys and
read-only socket observation without injection, seed search, retries or extended budgets.
