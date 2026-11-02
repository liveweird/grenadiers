# Punch

## Scenario: Punch wraps one retained bomb on contact and preserves its fuse

Two Chromium contexts use the pinned Punch arena with only Punch enabled and a 300-tick fuse.
The English controls help has five remappable actions, no Equipment row, and explains that Punch
activates by walking into a bomb. English desktop and Polish narrow clients reveal and collect
Punch, clear its route, and retain the existing Punch inventory and status presentation.

The holder plants a timed bomb at `(4,1)`, walks east to `(5,1)`, and moves west into it. Both
public streams observe the same bomb move along its three-cell wrapped path to `(1,1)` without a
manual Equipment command, placement event, detonation event, actor displacement, or private Punch
cooldown/history fields. Its identity, owner, range, placement tick, timed fuse, and standard blast
remain unchanged. Both clients capture the destination and vacated source cells, then observe the
same-ID natural detonation at `placedAt + 300`.

The holder repeats the contact punch with a fresh bomb and reconnects through the application
transport. The admitted baseline retains the room, seat, Punch inventory, bomb identity and fuse,
and reports only the preceding Bomb action ID. Pressing default E emits no command. Both clients
then observe the second bomb's natural detonation. The scenario uses UI actions, native keys, and
read-only socket observation without direct gameplay command injection.
