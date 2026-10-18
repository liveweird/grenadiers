# Invisible Bombs

## Scenario: Invisible Bombs hides live bombs and cues until exact expiry

Two real Chromium contexts use the repeating-byte-31 generated-v2 fixture. Player One reveals the
pickup at 3,1 with ordinary movement and a 300-tick bomb, escapes, and waits for fire to clear.
English desktop and Polish 320-pixel reduced-motion screenshots capture the exposed pickup before
collection.

After collection, both players place and escape from real bombs. Player Two's stream sees both
current bomb records and captured cues. Player One's stream keeps truthful live-bomb counts but
receives an empty bomb list and no post-collection placement or detonation events. Compositor tile
crops distinguish the sighted bomb from the blinded floor while public flames remain visible.

Player One reconnects while blinded and a bomb is live. The event-free admission preserves the
original physical expiry deadline. After the initial fire clears, Player Two places a late bomb in
the `T+390..T+470` window. The last paired publication at `T+597..T+599` keeps it hidden from
Player One and visible to Player Two. The first paired publication at `T+600..T+602` reveals
the same ID and cell with equal current remaining fuse on both views, without a replayed
placement cue. Pure-core tests establish the exact domain expiry at `T+600`. The route uses normal controls, no
state injection, seed search, retries, duration override or extended timeout.
