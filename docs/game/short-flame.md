# Short Flame

Short Flame is a configurable timed curse introduced in the v0.3.29 candidate. New rooms enable
it with every available item; change it in shared lobby settings before readiness. Classic settings
disables all optional items. See [verification](../verification.md) for
acceptance status.

## Collection and blast range

Ordinary explosions reveal the concealed pickup. Collection grants ten seconds
(600 physical ticks) of range-one bombs: the center and up to one adjacent cell in each
cardinal direction. Walls, blocks, other bombs and chains keep their ordinary behavior.
Recollection refreshes the deadline. This curse changes neither fuse timing nor damage.

Each successful placement captures its range once. Existing bombs keep their original range
when the curse is collected, refreshed or expires. Effects expire before placement at the
exact deadline; collection follows placement, so an earlier same-tick placement keeps its
previous range. Blast Range upgrades continue improving the base range underneath the curse.

## Other effects and presentation

Short Flame overrides Golden Flame while both are active. Their timers remain independent:
if Short Flame expires first, new bombs regain Golden Flame's range eight; otherwise new
bombs remain range one until Short Flame expires. Short Fuse independently modifies timing.
Shield, Heart equipment and damage grace remain separate; Short Flame provides no protection.

The HUD displays the timer alongside other effects. A distinct pixel pickup and actor cue
identify the curse; reduced motion keeps the cue static. Death and a new round clear it.
Reconnect restores remaining physical time without refreshing the deadline or replaying
collection effects. Result screens freeze the physical clock.

## Generation and compatibility

The enabled weight is 15, appended after Short Fuse. All sixteen previous pool masks retain
their exact generation paths. All thirty-two subsets of the five items available at
introduction remain valid; see the [current catalog](settings.md) for later items.
Rules version 5, generator version 2 and journal format 2 remain unchanged. The active
wire contract is [gameplay-v11](protocol-v11.md).
Deploy the paired server/browser build together: older clients require a reload for the current
wire contract and availability list. Retain a compatible server to replay newly enabled masks.
