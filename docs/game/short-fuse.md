# Short Fuse

Short Fuse is a configurable timed pickup. New rooms enable
it with every available item; change it in shared lobby settings before readiness. Classic settings
disables all optional items. Its release record is in
[item acceptance history](../history/game-acceptance.md#expanded-item-delivery-sequence).

## Collection and bomb timing

Ordinary explosions reveal the concealed pickup. Collecting it grants ten seconds
(600 physical ticks) of halved bomb fuses. Recollection refreshes the deadline. For each
successful placement, the server captures half the configured fuse, rounded up to the next
whole tick. For example, a three-second fuse becomes 90 ticks (1.5 seconds); a 1.5-second
fuse becomes 45 ticks (0.75 seconds).

A planted bomb keeps its captured deadline after the buff expires. Bombs planted before
collection keep their original timing. Effects expire before placement at their exact deadline;
collection happens after placement, so a same-tick placement does not receive the new buff.
Chain reactions still detonate bombs early. Short Fuse changes neither range nor damage.

## Independent effects and presentation

Short Fuse coexists with Shield, Golden Flame, Heart equipment and damage grace. Its timer
uses the physical simulation clock, freezes on results, and clears on death or a new round.
Reconnect restores the remaining timer and existing bomb deadlines without replaying collection.
The HUD, distinct pixel pickup and actor indicator identify the buff; reduced motion keeps
its indicator static.

## Generation and compatibility

The enabled weight is 15, appended after Golden Flame. Earlier item-pool masks retain their exact
generation paths; the [current catalog](settings.md) defines the complete set.

Rules version 5, generator version 2 and journal format 2 remain unchanged. The active
wire contract is [gameplay-v11](protocol-v11.md).
Deploy the paired server/browser build together: older clients require a reload for the current
wire contract and availability list. Bomb projections accept either the ordinary configured
fuse or its captured halved value when Short Fuse is enabled, independently of the owner's
current effect. Arbitrary durations and overdue deadlines are rejected.
