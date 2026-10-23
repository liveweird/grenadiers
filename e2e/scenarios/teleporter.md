# Teleporter

## Scenario: Teleporter swaps safe players, survives reconnect, and is consumed when unsafe

Two Chromium contexts use the repeating-byte-31 generated-v2 fixture with only Teleporter enabled.
Player One reveals the pickup at 3,1 with ordinary movement and a real bomb, escapes, and collects it
while moving east. Both views must publish the same new teleport marker, Player One lands at Player
Two's authoritative position, and Player Two lands on the collection cell.

Before collection, after the exposing bomb and flames have cleared, paired full-page captures record
the painted arena and visible pickup in desktop English and 320-pixel reduced-motion Polish layouts.
The mobile view must not overflow horizontally. These captures provide localized rendering evidence;
the paired position and marker assertions after collection prove the authoritative swap itself.

Player One then reconnects through the normal application transport. The admitted state retains the
same marker and position without replaying a swap. Player One reveals the pickup at 11,11 while
Player Two plants a real bomb at 3,1 and retains actor-specific exit grace. Player One collects the
second pickup with ordinary westward movement. The pickup event remains public and the pickup is
consumed, but neither actor swaps and the successful marker stays unchanged because Player One's
landing would overlap Player Two's bomb without owning that grace. The route uses no state
injection, seed search, retry, or timeout extension.
