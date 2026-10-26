# Heart pickup

Heart is included with every other configurable item in new-room defaults. Its release record is
in [item acceptance history](../history/game-acceptance.md#heart-v0325).

## Playing with Heart

New Expanded rooms enable every available item, including Heart. The room creator can disable it in the lobby
before becoming ready. Applying settings clears both players' readiness and publishes a new
settings revision. The **Classic settings** preset disables optional items and restores the
four original settings. Changes persist through the match and consensual rematches.

Heart can be concealed under a soft block. Destroy the block, wait for the fire to expire,
then walk over the exposed pickup. It occupies the equipment slot until consumed, replaced,
or cleared at death or a new round. Collecting a second Heart replaces the existing charge;
it never adds a second charge.

One lethal blast consumes Heart instead of eliminating the player. The player then receives
60 physical simulation ticks of damage grace: protection starts at the absorption tick and
expires immediately before damage at tick T+60. Overlapping flames and the two damage checks
within one update consume only one charge and produce one protection event. A Heart collected
during grace equips a future charge without extending the current grace period. Picking up
protection cannot undo damage that already occurred earlier in the update.

The HUD shows the equipped Heart and remaining grace. Reconnecting restores this current
state without replaying the previous absorption animation or sound. Shield is a separate timed buff and leaves the Heart slot intact. Active Shield prevents damage
before Heart is consumed. Other equipment, buffs, curses, weapons and monsters remain future work.

## Deterministic generation and compatibility

Rooms have immutable rules identity **Expanded v5** and map identity **generated_v2**, even
with Heart disabled. Generator v2 uses the configured enabled-item mask. Concealed pickups
retain the 30% placement chance and use weights 40 capacity, 40 range, 20 speed, plus 25 Heart
when enabled. Symmetric placement and spawn protection remain enforced.

With an empty enabled mask, generation matches generator v1 for the same seed, including
terrain, blocks and concealed upgrades; only its recorded generator identity differs.
Generator v1, earlier rules and archived replays retain their original behavior. Future
disabled items must introduce neither random draws nor pickup weights.

Gameplay v9 introduced the identities and the live [v11](protocol-v11.md) carries them. Other
socket versions are rejected before credential claim. Legacy state and event projections reject Heart instead of omitting or relabeling it.
Private replay format 2 records Expanded v5/generator v2 with the existing lifecycle policy;
complete settings already carry the enabled mask, so generation records need no extra field.

## Verification requirements

Release evidence must cover normal generated reveal/collection, non-stacking replacement,
both damage passes, overlapping flames, exact grace expiry, tick overflow, reconnect, round
reset, settings authorization, deterministic replay and empty-mask generation equivalence.
The native Heart scenario must use production generation and ordinary player controls,
without an inventory injection endpoint. Existing seeded scenarios explicitly choose the
Classic settings preset and retain their gameplay assertions and timing budgets.
