# Punch

## Scenario: Punch wraps one retained bomb three cells without extra cues and preserves its fuse

Two Chromium contexts use the repeating-byte-31 generated-v2 fixture with only Punch enabled and a
300-tick fuse. Before joining, the English desktop client remaps Equipment from E to F and uses that
actual key throughout play. The English desktop and Polish 320-pixel reduced-motion clients ready,
reveal and collect Punch with an ordinary bomb, then use two more ordinary bombs to clear the route.
A demo bomb is planted at `(4,1)` and the holder fully exits east to `(5,1)`, faces west, and presses
F. Both public streams observe the same entity ID move directly by its three-cell wrapped path to
`(1,1)`, with unchanged owner, range, placement tick, Timed trigger and Standard blast. The action
uses the immediate successor of the demo placement's shared manual-action ID, creates no placement
or detonation event, and leaves the actor position, heading and Teleporter epoch unchanged. Neither
public stream contains private Punch cooldown or bomb-history fields.

The holder escapes through `(4,1)` and `(3,1)` into safe cell `(3,2)` before either screenshot or
compositor crop. Both clients capture the punched bomb at `(1,1)` and the unobscured vacated source
cell `(4,1)`, then observe its natural same-ID detonation at `placedAt + 300`. Both recipient views
show both actors alive.

After that visual proof completes and the arena clears, the holder repeats the same route with a
fresh bomb. While this second punched bomb remains live at `(1,1)`, the English client closes its
actual application transport and resumes through the normal client path. The new admitted baseline
retains the room, seat, both living actors, Punch inventory and the same bomb identity, cell, owner,
range, placement tick, Timed fuse and Standard blast without private Punch fields or replayed action
events. The next single press of the remapped F key uses the successor action ID. The holder escapes,
both clients observe the second bomb's natural same-ID detonation at `placedAt + 300`, and a later
admitted baseline confirms that action was accepted without repetition. The scenario uses ordinary
UI actions, native keys and read-only socket observation without injection, seed search, retries,
direct gameplay commands or extended budgets.
