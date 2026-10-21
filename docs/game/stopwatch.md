# Stopwatch

Stopwatch is an opt-in Expanded-v5 pickup active for 180 physical ticks. Recollection refreshes
the holder's deadline. While a living actor holds it, the opponent's voluntary movement and manual
actions are frozen. Network input is still admitted and fresh direction state replaces old state;
manual actions attempted while frozen are consumed rather than replayed after thaw. Two holders
freeze each other until their independent deadlines expire. Death clears the effect.

Every Timed bomb pauses once per physical update while any living Stopwatch holder exists. A bomb
placed during a pause keeps its complete captured fuse on the placement update. Remote and Contact
bombs remain untimed, and fire, chains, damage, effect timers, round deadlines, automatic Diarrhea
placement, publication and reconnect clocks continue. Each Timed bomb privately retains its
original captured fuse and accumulated paused ticks so live and historical wire facts cannot alias
an ordinary schedule. A newly placed bomb accumulates only the remaining active Stopwatch window,
from its placement through the holder's fixed deadline. All pause arithmetic is checked before the
update mutates state.

Gameplay v9 publishes `voluntaryActionsFrozen` on visible actors and a pause flag on Timed bombs.
The flag is derived from both complete actors before Cloak recipient filtering; hidden actors stay
exactly identity, seat and alive status. Event audiences remain fixed when emitted. Gameplay v8 is
archived at the fourteen-item Cloak catalog and rejects Stopwatch, current pause and resumed pause
history even when a bomb is no longer paused.

Version 0.3.40 adds Stopwatch as the fifteenth item and final generator weight 15. The all-items
pool grows from 320 to 335 entries. Every earlier 16,384-mask path is unchanged, while gameplay v9
supports all 32,768 subsets. Rules version 5, generator version 2 and journal format 2 are unchanged.
