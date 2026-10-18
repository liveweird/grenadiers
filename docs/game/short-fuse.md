# Short Fuse

Short Fuse is an opt-in timed pickup in the v0.3.28 candidate. Enable it in shared lobby
settings before readiness. Heart remains the only optional item enabled by default;
Classic settings disables all optional items. See [verification](../verification.md) for
acceptance status.

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

The enabled weight is 15, appended after Golden Flame. All eight previous item-pool masks
retain their exact generation paths. All sixteen subsets of Heart, Shield, Golden Flame and
Short Fuse are supported; the default remains Heart-only.

Rules version 5, generator version 2, gameplay-v8 and journal format 2 remain unchanged.
Deploy the paired server/browser build together: older v8 clients reject the extended
availability list and need a reload. Bomb projections accept either the ordinary configured
fuse or its captured halved value when Short Fuse is enabled, independently of the owner's
current effect. Arbitrary durations and overdue deadlines are rejected.
