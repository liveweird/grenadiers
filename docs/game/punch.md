# Punch

Punch is opt-in permanent equipment in Expanded-v5 rooms. Press the remappable Equipment key to
punch the bomb in the adjacent cell indicated by the actor's authoritative heading. The target cell
does not wrap. Timed or Remote bombs with Standard, Piercing or Power blasts are eligible, including
a bomb whose Kick motion is active; Contact mines are excluded. A source touching active fire, a
Timed bomb already due after pause preflight, or a Remote bomb reserved for same-update detonation
cannot be punched. Failed, blocked, frozen, unequipped and cooling-down actions are gameplay no-ops
after normal server admission.

An eligible bomb searches exactly three interior cells at a time along the facing axis, wrapping
the playable columns 1–13 or rows 1–11. It takes the first floor cell without a soft block, barrel,
other bomb or positive-area overlap with a living actor. Search stops after one complete axis cycle,
and returning to the source fails unchanged. The logical hop is instantaneous: intermediate cells
do not cause fire, mine, collision or pickup interaction. Landing on fire is legal and participates
in the update's one later explosion plan.

Success preserves the bomb's entity ID, owner, range, blast, placement tick, trigger and captured
fuse. It stops active Kick motion, retains Kick history, removes every actor's grace for the moved
bomb and creates no placement, movement or sound event. Success at physical tick T blocks another
success through T+11 and permits one at T+12. The cooldown and persistent `hasBeenPunched` facts are
private trusted state retained through replacement, reconnect, death and results, then cleared by a
new round. No countdown, target highlight, blocked reason or success cue is published.

Version 0.3.45 added Punch to its twenty-item catalog. Its capability bit
is `2`; `ALL` is `3,145,723`. Generator v2 appends weight 15 after Kick, making the complete pool
410 and the Punch interval `[395,410)`. A Punch-only pool is 115. All 1,048,576 catalog subsets are
validated. Archived gameplay v9 and earlier reject Punch catalog entries, equipment, pickups,
cooldown and live or historical bomb provenance. This document describes reviewed source; native
same-image acceptance, accepted v0.3.44 parent integration and release publication remain pending.
