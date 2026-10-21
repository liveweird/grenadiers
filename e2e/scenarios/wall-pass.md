# Wall Pass

## Scenario: Wall Pass crosses one intact block, rejects planting and hard walls, and resumes inside

Two Chromium contexts use the repeating-byte-31 generated-v2 fixture with only Wall Pass enabled.
The English desktop and Polish 320-pixel reduced-motion clients ready before play, reveal the
pickup with an ordinary bomb, and move the holder into the intact block at `(3,2)`. Both public
views retain the opaque block and show the current equipment cue without exposing concealed
contents. A bomb action inside creates no bomb or event and consumes no inventory. Movement west
clamps at the hard wall `(2,2)`. A strictly newer ordinary reconnect admission retains the
equipment and position, after which normal movement exits north. The scenario uses native keys,
ordinary UI actions, and read-only socket observation. A legal bomb placed after exit has the
immediate successor of the reveal bomb's entity ID, proving the rejected inside action consumed no
entity allocation. Equipment-loss exit grace remains covered
by core and protocol tests because the sole-item fixture has no replacement pickup.
