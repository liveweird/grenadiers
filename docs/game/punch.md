# Punch

## Contact activation

Punch is a configurable permanent equipment item. In current gameplay, walk into an adjacent
bomb to punch it in the physical movement direction. No Equipment key is needed. A positive
forward request reaching the bomb’s boundary counts as contact; stationary touching, turn
centering, forced shoves, frozen actors and leaving an own bomb under placement grace do not.
A nearer or equally blocking player/wall prevents the attempt.

Each exact bomb receives at most one attempt during uninterrupted contact, including failed
landing or cooldown attempts. Releasing/repressing a key, changing heading, freezing or replacing
equipment does not rearm that contact. Separate physically before trying again. Multiple touching
bombs retain independent contact latches; removal, teleport, death and a new round clear the
appropriate contact state.

Timed or Remote bombs with Standard, Piercing or Power blasts are eligible, including rolling
bombs. Contact mines are excluded. Active source fire, a timed fuse already due, or a remote
reserved for same-update detonation prevents Punch. Contacts resolve after manual bomb actions
and before explosions, rechecking the exact bomb and actor. The actor does not advance into the
vacated tile during that update.

## Flight and cooldown

The bomb searches three interior cells at a time along the contact axis, wrapping around the
playable board edges. It takes the first floor cell without a soft block, unlit barrel, another
bomb or overlap with a living actor. Search ends after one axis cycle; returning to the source
fails unchanged. Intermediate cells cause no collision, fire, mine or pickup interaction.
Landing on fire is legal and participates in the same update’s explosion plan.

Success preserves entity ID, owner, range, blast, placement tick and fuse/trigger. It stops Kick
motion, retains Kick history and removes placement grace for the moved bomb. Success at tick T
permits another success at T+12. Cooldown and Punch provenance remain private; no separate
Equipment action, target marker or blocked-reason message is published.

## CPU and compatibility

Current CPUs refuse their own forward contact when relocation is absent from their certified
route; they retain useful wall approaches and own-bomb departures. This does not guarantee
survival against arbitrary rival actions. Changed public bomb facts invalidate conditional routes.

Gameplay20 selects contact activation; archived gameplay19 and replay30 retain manual behavior.
Private replay31 records activation version2 explicitly. Earlier contracts keep their original
catalog boundaries; gameplay9 and earlier reject Punch. See the [protocol](protocol-v20.md),
[controls](controls.md) and [original manual acceptance](../history/game-acceptance.md#punch-v0345).
