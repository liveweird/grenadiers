# Invisible Bombs

## Scenario: Invisible Bombs hides live bombs and cues until exact expiry

Two real Chromium contexts use the repeating-byte-31 generated-v2 fixture. Player One reveals the
pickup at 3,1 with ordinary movement and a 300-tick bomb, escapes, and waits for fire to clear.
English desktop and Polish 320-pixel reduced-motion screenshots capture the exposed pickup before
collection.

Player Two prepositions at 12,11 before Player One collects. Player One immediately plants at 3,1,
Player Two plants sequentially at 12,11, and the players escape along independent routes in
parallel. The first bomb's placement plus its exact 300-tick fuse fits before `T+600`. Player Two's
stream sees both current bomb records and captured cues. Player One's stream keeps truthful
live-bomb counts but receives an empty bomb list and no placement or detonation events whose event
ticks precede the exact expiry. Player Two's sighted stream witnesses a real pre-expiry detonation,
so the cue filtering proof is non-vacuous.

Player One reconnects immediately after both live bombs are observed, while still blinded. A raw
post-disconnect admission from the current round has no bombs, retains the positive effect, and
preserves the original physical expiry deadline exactly. Subsequent publications keep the bomb
hidden from Player One and show Player One's original live timed bomb to Player Two. Parallel,
viewport-bounded 3,1 tile captures are bracketed by those live-bomb and deadline proofs. After the
bombs and public flames clear, each capture is compared with a clean tile from the same page:
Player One's hidden tile matches and Player Two's sighted tile differs. The sighted stream also
supplies the awaited pre-expiry detonation witness. The mirrored block then exposes the same pickup
at 11,11. Player Two collects it, and both streams derive the same new `T2+600` deadline from the
collection event and remaining effect ticks.

Player Two then places a late bomb in the unchanged `T2+390..T2+470` window. Player One is the
sighted recipient and receives that bomb's placement cue; newly blinded Player Two receives no
placement cue while blinded. The last paired publication at `T2+597..T2+599` keeps the bomb hidden
from Player Two and visible to Player One. The first paired publication at `T2+600..T2+602`
reveals the same ID and cell with equal current remaining fuse on both views, without replaying the
placement cue to Player Two. Pure-core tests establish exact domain expiry at 600 ticks. The route
uses normal controls, no state injection, seed search, retries, duration override or extended timeout.
