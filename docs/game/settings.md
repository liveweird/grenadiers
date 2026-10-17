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

Both players see accepted values. Editing requires the original creator to be connected,
unready and in the waiting lobby. Apply changes to update the shared rules; a genuine change
clears both players' readiness. Restore defaults prepares the original values for applying.
Once a match starts, settings stay locked through all automatic rounds. Both players can
consent to a rematch and then edit in the unready lobby again.

Reconnect preserves creator authority while the seat remains reserved. If the creator leaves
permanently or the reservation expires, the last accepted configuration remains; create a new
room to edit again. A replacement occupant does not become creator.

Local keyboard and sound preferences are independent. Turn assistance still uses the normal
movement budget and swept collision/fire checks. Extra configuration for arena generation,
round/flame duration, loadouts and item pools remains in the [backlog](backlog.md).
