# Monster lairs

## Scenario: enabled lairs keep one mob each, render on both clients, reconnect and rematch

The dedicated suite requires the published `arena-v3-lairs` fixture acknowledgement. English desktop and Polish
320-pixel reduced-motion clients join one real two-human room. The creator disables
the storm cloud, sets lairs to the five-second interval and one living mob per lair, then both players start a
rules9 round. Both public streams expose the two fixed lair pillars and the same two mobs without target or path
internals. The mobs move authoritatively, retain the same IDs and suspended/null timers through another full respawn interval, and change pixels in the
live Pixi canvas while the lair artwork differs from an ordinary wall pillar. While the phase is still Running,
the mobile transport reconnects normally with populated state. The clients then cross another full spawn
interval without another birth and capture both localized rendered views before a native bomb finishes the one-win
match. A consensual rematch retains the lair tuning and creates the two lairs again.

## Scenario: disabled lairs publish empty lair and mob arrays

Two real browsers create and join another room through the application UI. **All off** disables items, the storm
cloud, and lairs while retaining the 10-second respawn delay and fixed capacity of one. The accepted rules9 room starts via
native ready actions and publishes empty lair and mob arrays to both recipients.
