# Room gameplay settings

Implemented in v0.3.17; see [ADR 0020](../decisions/0020-shared-gameplay-settings.md) and
the [verification record](../verification.md) for accepted deployment evidence.

The creator can configure these rules before either starts the match:

| Control | Choices | Effect |
| --- | --- | --- |
| Starting speed | 3–5 cells/second, steps of 0.5 | Sets both players' starting speed; speed pickups still cap at 5. |
| Corner tolerance | 1/16–¼ tile, steps of 1/16 | Controls how far from a corridor center a perpendicular turn may align. |
| Wins to finish | 1–9 | Sets round wins needed for match victory. Draws award no point. |
| Bomb fuse | 1–5 seconds, steps of 0.5 | Sets the delay for successfully planted ordinary bombs. |
| Heart | Enabled or disabled; enabled by default | Adds the one-hit protection pickup to generated soft blocks. |
| Shield | Enabled or disabled; disabled by default | Adds five-second immunity pickups; recollection refreshes the timer. |
| Golden Flame | Enabled or disabled; disabled by default | Adds ten-second maximum-range pickups; recollection refreshes the timer. |
| Short Fuse | Enabled or disabled; disabled by default | Adds ten-second pickups that halve newly planted bomb fuses; recollection refreshes the timer. |
| Diarrhea | Enabled or disabled; disabled by default | Adds ten-second curses that automatically attempt one ordinary bomb placement per physical tick. |
| Slow foot | Enabled or disabled; disabled by default | Adds independently timed ten-second stacks that each halve movement speed, up to six stacks. |
| Piercing Bomb | Enabled or disabled; disabled by default | Stores one special manual bomb whose blast crosses breakable blocks. |
| Power Bomb | Enabled or disabled; disabled by default | Stores one special manual bomb whose diamond-shaped blast ignores obstruction while leaving hard walls intact. |
| Remote Bomb | Enabled or disabled; disabled by default | Stores one manual bomb with no fuse; the owner's next bomb action detonates the armed bomb. |
| Short Flame | Enabled or disabled; disabled by default | Adds ten-second curses that limit new bombs to range one, overriding Golden Flame. |
| Long Fuse | Enabled or disabled; disabled by default | Adds ten-second curses that double new ordinary timed bomb fuses. Short and Long Fuse cancel while both are active. |
| Reverse Controls | Enabled or disabled; disabled by default | Adds ten-second curses that reverse held movement directions. |

Both players see accepted values. Editing requires the original creator to be connected,
unready and in the waiting lobby. Apply changes to update the shared rules; a genuine change
clears both players' readiness. Restore defaults prepares Expanded defaults, including Heart.
Classic settings prepares the original four values with optional items disabled. Both presets
require Apply rules before changing the shared configuration.
Once a match starts, settings stay locked through all automatic rounds. Both players can
consent to a rematch and then edit in the unready lobby again.

Reconnect preserves creator authority while the seat remains reserved. If the creator leaves
permanently or the reservation expires, the last accepted configuration remains; create a new
room to edit again. A replacement occupant does not become creator.

Local keyboard and sound preferences are independent. Turn assistance still uses the normal
movement budget and swept collision/fire checks. Extra configuration for arena generation,
round/flame duration, loadouts and additional item pools remains in the [backlog](backlog.md).

Heart is introduced by the v0.3.25 implementation candidate. Expanded rooms retain immutable
rules version 5/map `generated_v2` when Heart is disabled, including after preset changes.
Shield is available separately in the v0.3.26 candidate, Golden Flame in v0.3.27, Short Fuse in v0.3.28, Short Flame in v0.3.29, Long Fuse in v0.3.30, Reverse Controls in v0.3.31, Diarrhea in v0.3.32, Slow foot in v0.3.33, Piercing Bomb in v0.3.34, Power Bomb in v0.3.35, and Remote Bomb in v0.3.36; all twelve switches preserve one another.
See [Heart behavior](heart.md), [Remote Bomb](remote-bomb.md), [Piercing Bomb](piercing-bomb.md), [Power Bomb](power-bomb.md), [Shield behavior](shield.md), [Golden Flame](golden-flame.md), [Short Fuse](short-fuse.md), [Short Flame](short-flame.md), [Long Fuse](long-fuse.md), [Reverse Controls](reverse-controls.md), [Diarrhea](diarrhea.md), [Slow foot](slow-foot.md), and [protocol compatibility](protocol-v8.md).
