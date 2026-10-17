# ADR 0020: Shared, revision-bound room settings

Status: implementation candidate. Depends on classic-v3 turning and gameplay v5.

## Scope and defaults

The first settings panel exposes starting movement speed, corner alignment tolerance, wins
needed to finish a match and ordinary bomb fuse time. These directly affect playability while
retaining the current arena, 180-second round, upgrade caps and fixed 60 Hz simulation.

| Setting | Valid choices | Default |
| --- | --- | --- |
| Starting speed | Existing levels 0–4: 3, 3.5, 4, 4.5, 5 cells/second | 3 cells/second |
| Turn tolerance | 64, 128, 192 or 256 of 1,024 units per tile | ¼ tile |
| Wins to finish | Integers 1–9 | 3 |
| Fuse | 60–300 ticks, in 30-tick steps | 3 seconds |

Zero tolerance is excluded: fractional speeds can step across a corridor center without ever
landing exactly on it. The 64-unit minimum covers the nearest sampled center at all supported
speeds. These initial bounds are tuning choices, not historical Bomberman constants. Draws award no
point and do not consume a fixed round allowance. Starting speed does not change the existing
5-cells/second upgrade cap. Other requested settings follow as separately validated increments.

## Authority and locking

The original creator edits only while connected, unready and in Waiting. The opponent may
already be ready. A complete, valid, changed configuration clears both readiness flags;
identical updates preserve readiness. Settings remain fixed through the match's automatic
rounds. A mutual rematch retains the settings and returns players to an unready lobby.

Editor identity belongs to the original reserved seat, surviving same-process reconnect and
connected takeover. Permanent creator departure/expiry removes editing authority; a later
occupant of that seat does not inherit it. Host transfer is deferred. Everyone sees accepted
settings, their revision and whether an editor remains.

Updates carry the expected settings revision alongside existing room/epoch/sequence binding.
Ready commands carry the settings revision being accepted, preventing an in-flight old Ready
from starting a match after a configuration change. Rejected updates have no mutation; check
counter bounds before changing state. Successful no-ops still settle the requester's UI.

## Identity, transport and evidence

Gameplay v6 adds the complete closed settings object, revision/editor fields, an atomic
update command and revision-bound readiness. Maximum structural scores become nine; semantic
validation compares scores and winners against the selected target. V5 stays archived and
unsupported clients are rejected before credential binding.

Configurable rules version 4 stores the complete immutable settings identity. Constructors
apply speed to starting actors and fuse/tolerance to actual physics; series scoring uses the
selected target. Both snapshot publication and personal reconnect admission check the actual
round configuration against the room. Legacy rules and replay journals keep their own values.

Replay captures accepted settings changes and authority release, retaining explicit initial
settings and lifecycle identity. No persistence, global settings, hidden map information or
new infrastructure is introduced. Reconnect remains process-local; deployment ends matches.

Acceptance covers invalid bounds/steps, stale updates/readiness, editor lifetime and seat
reuse, automatic rounds/rematches, speed/fuse outcomes, target-one/target-nine scoring,
forged binding and exact replay. Two native browsers must agree on settings and their effects;
full gates and same-image Compose/Kubernetes acceptance remain required before release.
