# Oil Barrel

Oil Barrel is an opt-in arena object in Expanded-v5 rooms. It is disabled by default. Enabling it
allows generated-v2 arenas to reserve at most eight Oil sites for the whole round; the object is
never inventory, a timed effect, or a collectible pickup.

A site begins concealed behind a generated soft block. After that block is destroyed, revelation
waits until the start of an update where the floor cell has no block, bomb, fire, pickup, other
object, or positive overlap with a living actor. Until then the pending site remains private. A
revealed unlit barrel is solid to actors and bomb placement. Wall Pass may cross it, using private
exit grace if the equipment is lost while overlapping it, but Wall Pass never prevents fire damage.

An explosion that reaches an unlit barrel ignites it without adding an explosion origin or extra
rays. Standard rays stop at the barrel as it existed at the start of the explosion plan. Piercing
rays continue after spending that cell, while Power Bomb keeps its Manhattan footprint. The barrel
then becomes passable fire for the rest of the active round. Its fire participates in movement,
bomb, Teleporter, forced-movement, and damage rules, but it has no finite expiry. Terminal frames
retain the frozen burning board; the next round creates a new generated world.

Gameplay v11 publishes only materialized sites in the required sorted `oilBarrels` array, with
state `unlit` or `burning`. Concealed and pending locations, typed exit grace, counts, seeds and
generation records remain private. Burning Oil is distinct from finite `flames`; clients must not
invent an expiry, pickup event, ignition event, countdown, or sound from an array transition.

Version 0.3.47 adds Oil Barrel at capability bit 20 (`1,048,576`) immediately before Teleporter.
The 22-item catalog accepts all `4,194,304` subsets. Generation appends Oil's weight-15 interval
`[425,440)` after Bracelet; an Oil-only pool is 115. Whole mirrored placements are accepted only
when the lifetime eight-site cap permits them. Rejected over-cap draws keep their soft blocks and
consume no replacement draw. Oil-disabled masks retain their prior generated output.

See [settings](settings.md), [generation](generation.md), [gameplay v11](protocol-v11.md), and the
[production-seed test](../testing.md#oil-barrel-production-seed-scenario).
