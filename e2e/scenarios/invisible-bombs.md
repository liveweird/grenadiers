# Invisible Bombs

## Scenario: Invisible Bombs hides live bombs and cues until exact expiry

Two real Chromium contexts use the repeating-byte-31 generated-v2 fixture. Player One reveals the
pickup at 3,1 with ordinary movement and a 300-tick bomb, escapes, and waits for fire to clear.
English desktop and Polish 320-pixel reduced-motion screenshots capture the exposed pickup before
collection.

After collection, both players place and escape from real bombs. Player Two's stream sees both
current bomb records and captured cues. Player One's stream keeps truthful live-bomb counts but
receives an empty bomb list and no placement or detonation events whose event ticks precede the
exact `T+600` expiry. Player Two's sighted stream witnesses a pre-expiry detonation, so the cue
filtering proof is non-vacuous. Compositor tile crops distinguish the sighted bomb from the blinded
floor while public flames remain visible.

Player One reconnects while blinded and a bomb is live. The event-free admission preserves the
original physical expiry deadline. After the initial fire clears, the mirrored block exposes the
same pickup at 11,11. Player Two collects it, and both streams derive the same new `T2+600`
deadline from the collection event and remaining effect ticks.

Player Two then places a late bomb in the unchanged `T2+390..T2+470` window. Player One is the
sighted recipient and receives that bomb's placement cue; newly blinded Player Two receives no
placement cue while blinded. The last paired publication at `T2+597..T2+599` keeps the bomb hidden
from Player Two and visible to Player One. The first paired publication at `T2+600..T2+602`
reveals the same ID and cell with equal current remaining fuse on both views, without replaying the
placement cue to Player Two. Pure-core tests establish exact domain expiry at 600 ticks. The route
uses normal controls, no state injection, seed search, retries, duration override or extended timeout.
