# Golden Flame

Golden Flame is a configurable timed pickup. New rooms
enable it with every available item; the creator can change the draft in shared lobby settings
before readiness, while Classic settings clears every optional item. Its release record is in
[item acceptance history](../history/game-acceptance.md#expanded-item-delivery-sequence).

## Collection and blast range

The pickup follows ordinary hidden-block, pending, exposed and collected phases. Collection
gives ten seconds (600 physical ticks) of maximum blast range, eight cells per ray. Recollection
refreshes its deadline. Hard walls, soft blocks and other bombs retain their ordinary blocking
and chain behavior; this is a range buff rather than a piercing or area weapon.

Each successful placement captures the active range. A bomb planted during the effect keeps
range eight after the effect expires. A bomb planted before collection keeps its original
range. Effects expire before placement at their exact deadline; collection happens after the
placement phase, so a same-tick placement does not receive a newly collected buff.

Blast Range upgrades continue improving the base range underneath the effect. The actor's
published base range stays unchanged by Golden Flame; planted bomb records contain the actual
captured range. When the effect expires, future bombs use the upgraded base range again.

## Independent effects and presentation

Golden Flame coexists with Shield, Heart equipment and damage grace. It provides no protection
against damage. Death and a new round clear it. Reconnect restores its remaining physical-clock
timer without replaying collection effects. Result screens freeze physical progression.

The HUD shows its current timer. A distinct pixel pickup and actor indicator identify it;
reduced-motion presentation keeps the indicator static. Server and browser must be upgraded
together: older clients require reload for the [current gameplay-v13 contract](protocol-v13.md).

## Generation and compatibility

The enabled Golden Flame weight is 15, appended to capacity 40, range 40, speed 20, enabled
Heart 25 and enabled Shield 15. Existing item-pool masks retain their exact generation path.
The current rules/generator identity remains Expanded v5/generated v2; gameplay-v13 and human-duel journal
format 2 represent the effect. Archived schemas and generator-v1 behavior remain frozen.
