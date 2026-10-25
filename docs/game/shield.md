# Shield pickup

Status: implemented; published in
[v0.3.26](https://github.com/liveweird/grenadiers/releases/tag/v0.3.26) as an opt-in pickup.

## Playing with Shield

The creator can configure Shield in the unready lobby, independently of Heart. Apply rules before
both players ready. New-room defaults enable both; Classic settings disables every optional item.
Settings remain immutable during play and persist through rounds/rematches.

Destroy a soft block containing Shield, wait for fire to clear, then walk over the exposed blue
shield. Collection grants **300 physical simulation ticks (five seconds)** of immunity. Another
Shield refreshes the deadline to collection tick + 300; it does not stack strength or add another
five seconds to the previous deadline. Equipment remains unchanged.

Shield expires before movement and damage at its exact deadline. It prevents ordinary blast
damage before Heart is considered; Heart is therefore retained. Existing Heart grace remains
valid independently. Each protected player produces at most one Shield prevention event per
physical tick, even with overlapping flames and both damage checks. Picking up Shield cannot
undo damage that occurred earlier in that update. Monsters remain future work; no monster
collision system is claimed.

The HUD shows the current effect and remaining time. A pixel ring distinguishes Shield from
Heart's corner-shaped grace markers. Reduced motion keeps the aura static. A reconnect restores
the current remaining timer without replaying old protection events. Death and round reset clear
both timed protection and equipment.

## Generation and compatibility

Shield uses bit 9 in the checked optional-item mask. Its initial release supported masks
0, 16 (Heart), 512 (Shield), and 528 (both); those masks retain their generation paths.
The [current catalog](settings.md) also supports later items. The historical empty and Heart-only
seeded generation paths remain unchanged.

With only Heart and Shield enabled, rules 5 / generator 2 sample weights 40 capacity,
40 range, 20 speed, 25 Heart and 15 Shield. Disabled items contribute no weight or additional random draw.
The live gameplay contract ([v11](protocol-v11.md)) and private journal format 2 are current;
older contracts and journal format-1 rules retain their original behavior. Checked timer overflow fails the step atomically.

See [the seeded browser scenario](../../e2e/scenarios/shield.md), [deployment reproduction](../deployment.md),
and [verification](../verification.md) for acceptance status and operational evidence.
