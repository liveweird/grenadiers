# Forgiving keyboard turns

## Scenario: native keyboard turns recover from a wall stop outside the legacy alignment window

Two independent rendered browsers join a random generated classic-v3 room. Native Left holds
player one's footprint against the west border at x=1,344, exactly 192 fixed-point units away
from the spawn corridor centerline. Native Down then aligns to x=1,536 and advances south
through the guaranteed opening. Both players observe the living actor's aligned position and
new heading. This real wall-stop case exceeds the legacy 128-unit window; the new 256-unit
window accepts it without manipulating positions, clocks, sockets or authoritative state.
