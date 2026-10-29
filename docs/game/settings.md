# Room gameplay settings

See [ADR 0020](../decisions/0020-shared-gameplay-settings.md) for the shared configuration model.
Historical activation details are in the [release record](../history/configuration-activation.md#room-settings-activation).

The creator can configure these rules before the match starts. New rooms use rules 7 and include
the environment setting `stormCloud` (enabled by default). All on and Restore enable the cloud;
All off and Classic disable it. Custom room creation separately
selects an immutable **Board size** / **Rozmiar planszy** of 11×9, 13×11 or 15×13 (default);
it is not an editable gameplay setting. Quick duel and Solo always use 15×13. See
[board geometry](board-sizes.md).

| Control | Choices | Effect |
| --- | --- | --- |
| Starting speed | 3–5 cells/second, steps of 0.5 | Sets every player's starting speed; speed pickups still cap at 5. |
| Corner tolerance | 1/16–¼ tile, steps of 1/16 | Controls how far from a corridor center a perpendicular turn may align. |
| Wins to finish | 1–9 | Sets round wins needed for match victory. Draws award no point. |
| Bomb fuse | 1–5 seconds, steps of 0.5 | Sets the delay for successfully planted ordinary bombs. |
| Heart | Enabled or disabled; enabled by default | Adds the one-hit protection pickup to generated soft blocks. |
| Bracelet | Enabled or disabled; enabled by default | Adds passive equipment that shoves a contacted rival up to one tile. |
| Punch | Enabled or disabled; enabled by default | Adds permanent equipment for moving the adjacent facing bomb to the next open three-cell wrapped landing. |
| Kick | Enabled or disabled; enabled by default | Adds permanent equipment for kicking stationary Timed and Remote bombs; Contact mines are excluded. |
| Wall Pass | Enabled or disabled; enabled by default | Adds permanent equipment for crossing breakable blocks; hard walls, bombs and damage remain effective. |
| Shield | Enabled or disabled; enabled by default | Adds five-second immunity pickups; recollection refreshes the timer. |
| Stopwatch | Enabled or disabled; enabled by default | Freezes the opponent's voluntary actions and pauses Timed bombs for three seconds. |
| Golden Flame | Enabled or disabled; enabled by default | Adds ten-second maximum-range pickups; recollection refreshes the timer. |
| Short Fuse | Enabled or disabled; enabled by default | Adds ten-second pickups that halve newly planted bomb fuses; recollection refreshes the timer. |
| Diarrhea | Enabled or disabled; enabled by default | Adds ten-second curses that automatically attempt one ordinary bomb placement per physical tick. |
| Slow foot | Enabled or disabled; enabled by default | Adds independently timed ten-second stacks that each halve movement speed, up to six stacks. |
| Piercing Bomb | Enabled or disabled; enabled by default | Stores one special manual bomb whose blast crosses breakable blocks. |
| Power Bomb | Enabled or disabled; enabled by default | Stores one special manual bomb whose diamond-shaped blast ignores obstruction while leaving hard walls intact. |
| Remote Bomb | Enabled or disabled; enabled by default | Stores one manual bomb with no fuse; the owner's next bomb action detonates the armed bomb. |
| Landmine | Enabled or disabled; enabled by default | Stores one passable manual mine that detonates on swept actor contact after initial exit grace. |
| Invisibility Cloak | Enabled or disabled; enabled by default | Hides the actor's position and gameplay details from the opponent for ten seconds. |
| Invisible Bombs | Enabled or disabled; enabled by default | Hides every bomb and its placement or detonation cue from the affected player for ten seconds. |
| Oil Barrel | Enabled or disabled; enabled by default | Adds at most eight arena objects that reveal solid and become passable fire until round end. |
| Teleporter | Enabled or disabled; enabled by default | Atomically swaps its collector with a randomly selected living rival when both destination footprints are safe; an unsafe pickup is consumed without movement. |
| Short Flame | Enabled or disabled; enabled by default | Adds ten-second curses that limit new bombs to range one, overriding Golden Flame. |
| Long Fuse | Enabled or disabled; enabled by default | Adds ten-second curses that double new ordinary timed bomb fuses. Short and Long Fuse cancel while both are active. |
| Reverse Controls | Enabled or disabled; enabled by default | Adds ten-second curses that reverse held movement directions. |

All players see accepted values. Editing requires the original creator to be connected,
unready and in the waiting lobby. Apply changes to update the shared rules; a genuine change
clears every human's readiness. **All on** and **All off** change the item switches and cloud
setting in the draft; they do not reset numeric rules or submit automatically. Restore defaults prepares the
numeric Expanded defaults with every currently available item enabled. Classic settings prepares
the original four values with optional items disabled. Every draft change requires **Apply rules**
before it changes the shared configuration.
Settings lock at first start and remain locked through automatic rounds and rematches. Create a
new room to change them.

Reconnect preserves creator authority while the seat remains reserved. If the creator leaves
permanently or the reservation expires, the last accepted configuration remains; create a new
room to edit again. A replacement occupant does not become creator.

Local keyboard and sound preferences are independent. Turn assistance still uses the normal
movement budget and swept collision/fire checks. Round/flame duration and loadouts are not
configurable here.

The map identity remains `generated_v2`. All 22 item switches preserve one another.
See [Oil Barrel](oil-barrel.md), [Heart behavior](heart.md), [Bracelet](bracelet.md), [Punch](punch.md), [Kick](kick.md), [Wall Pass](wall-pass.md), [Remote Bomb](remote-bomb.md), [Piercing Bomb](piercing-bomb.md), [Power Bomb](power-bomb.md), [Landmine](landmine.md), [Stopwatch](stopwatch.md), [Invisibility Cloak](invisibility-cloak.md), [Invisible Bombs](invisible-bombs.md), [Teleporter](teleporter.md), [Shield behavior](shield.md), [Golden Flame](golden-flame.md), [Short Fuse](short-fuse.md), [Short Flame](short-flame.md), [Long Fuse](long-fuse.md), [Reverse Controls](reverse-controls.md), [Diarrhea](diarrhea.md), [Slow foot](slow-foot.md), and [protocol compatibility](protocol-v11.md).
