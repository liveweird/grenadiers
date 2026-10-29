# Monster lairs

## Scenario: enabled lairs spawn capped private moving mobs, render on both clients, reconnect and rematch

The dedicated suite requires the published `arena-v3-lairs` fixture acknowledgement. English desktop and Polish
320-pixel reduced-motion clients join one real two-human room. The creator disables
the storm cloud, sets lairs to the five-second interval and one living mob per lair, then both players start a
rules8 round. Both public streams expose the two fixed lair pillars and the same two mobs without target or path
internals. The mobs move authoritatively, remain capped through another spawn deadline, and change pixels in the
live Pixi canvas while the lair artwork differs from an ordinary wall pillar. Both localized clients capture the
rendered state. The mobile transport reconnects normally with state, then a native bomb finishes the one-win
match. A consensual rematch retains the lair tuning and creates the two lairs again.

## Scenario: disabled lairs publish empty lair and mob arrays

Two real browsers create and join another room through the application UI. **All off** disables items, the storm
cloud, and lairs while retaining the 10-second / three-mob numeric defaults. The accepted rules8 room starts via
native ready actions and publishes empty lair and mob arrays to both recipients.
