# Diarrhea

Diarrhea is an opt-in timed curse in the v0.3.32 candidate. Collection activates or refreshes
it for exactly 600 physical simulation ticks. While active, each living actor contributes one
automatic ordinary-bomb intent on a tick that has no retained manual bomb request. Automatic
bombs capture the configured fuse, the actor's current derived range, and the actor as owner.
They never inspect or consume stored special weapons.

Collection occurs after placement, so the first automatic intent is on the following tick.
Expiry occurs before placement, so the collection tick plus 600 has no automatic intent. Death,
round completion, and round reset clear the effect. A disconnected actor whose seat remains
reserved stays in the simulation and continues producing intents.

## One placement resolution

| Retained request for a seat | Active curse | That seat's single intent |
| --- | --- | --- |
| Manual bomb | Either | Manual ordinary placement |
| None | Yes | Automatic ordinary placement |
| None | No | None |

Both seats' selected intents enter the existing eligibility and same-cell conflict resolver
together. Two eligible claims on one cell both fail. An ineligible claimant does not block an
eligible one. A failed manual request does not receive an automatic retry, duplicate manual
requests still produce one attempt, and `Clear` removes earlier manual requests without removing
the curse. Automatic intents do not enter the mailbox, consume an action ID, or advance the
reconnect action floor.

Placement follows movement and existing-fire damage, then explosions and final-fire damage.
Capacity released by an explosion can therefore be reused on the next physical tick. The option
is disabled by default; Heart-only defaults and Classic settings remain unchanged. Local checks
and independent reviews pass; native browser acceptance and final CI remain pending.
