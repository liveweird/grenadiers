# Headless participant model

The shared game core can construct two, three or four actors on the fixed 15×13 arena.
Live rooms, archived protocol versions 2–13, CPU observations and match lifecycle remain
limited to two participants. The [multiplayer design](multiplayer-design.md) defines the
larger live system; outstanding work belongs in the [backlog](../../BACKLOG.md).

## Construction and rules

`PlayerCount` accepts exactly 2–4. `State` constructors accept actor arrays or vectors,
validate unique actor IDs and seats, and store the canonical prefix One through the
selected count. Arena spawn count must match actor count. `actors()` returns a slice;
`actor(seat)` returns `None` for a nonparticipant.

Rules versions 1–5 accept only two actors. `RulesConfig::multiplayer_v6(settings)` supports
all 22 optional-item bits with two, three or four actors. Basic bomb capacity, flame range
and speed pickups remain available. A v6 profile containing Teleporter requires an explicit
`SimulationSeed`, through `State::new_at_tick_with_assignment_and_seed` or `State::new_round`.
Other profiles can use the existing seedless constructors; historical rules consume no
simulation randomness.

## Per-round arena and actions

Arena metadata describes one round's actual spawn cells. Two-player opposite-corner rules
are unchanged; three/four-player spawns must be distinct interior corners with clear inward
escape cells. `RoundAssignment` rotates seat assignment across the supplied cells and action
priority across participants. `spawn_for` returns `None` for a seat absent from that arena.

A three-player series must construct its arena metadata anew for each round to rotate the
unused fourth corner. Reusing the same three spawn cells does not perform that rotation.
The [version 3 generator](multiplayer-generation.md) makes this choice from the actual round
and reserves every corner escape. The state constructor does not rewrite caller terrain or
spawns. Historical generators retain their two-player behavior.

All input seats are validated before tick advancement or mutation. Simultaneous placement
at a shared cell rejects every contender; independent placements use canonical ordering.
Pickup ties use the rotated participant priority. Zero survivors draw, one wins, and multiple
survivors continue until the configured timeout. Actor-sized traces and event audiences
contain only actual participants; legacy `EventAudience::Both` still means One and Two.

## Equipment and Teleporter

Punch collects requests from one pre-mutation world. Contenders for the same bomb use rotated
round priority; different bombs sharing a landing cell are all rejected. Accepted relocations
commit together and clear moved-bomb grace from every actor.

Bracelet chooses each pusher's earliest contact, using round priority for ties. Only disjoint
pusher/victim pairs can succeed in one phase; this prevents recursive shove chains. Displacement
respects terrain, bombs, Oil and other living actors. Conflicting victim sweeps use the same
priority. Fire and mines inspect every accepted forced segment, including both independent shoves.

Teleporter resolves one collected request per tick in round priority and chooses among canonical
living opponents, including cloaked opponents. Zero/one candidates need no random draw; multiple
candidates use the independently seeded ChaCha20 stream with unbiased, bounded rejection sampling.
Unsafe or equal-position selections consume the pickup without retrying; draw exhaustion also
produces no swap. Whole-step rollback preserves the stream. Seeds and RNG internals are redacted
from debugging and never appear in public state.

## Participation across rounds

`RoundActorInit` pairs an ordinary actor initializer with a participation flag. `State::new_round`
validates the complete original roster and requires at least two participating actors. Inactive
participants remain canonical dead actors with empty controls, effects, inventory and grace. Use
lifecycle eligibility for this flag: ordinary deaths may respawn next round; permanent departures
must remain inactive.

`State::eliminate_departed([bool; 4])` validates unused mask entries before mutation, bypasses
damage protection, retains owned bombs and emits elimination/result events at the current tick.
It does not advance the clock, running updates, world or RNG. Append its events to the already
completed report for that tick. Already-final physical states remain frozen. Runtime must settle
physics/scoring before departures and complete any departure-driven result without a second tick.

## Archived compatibility

Historical wire seats remain One/Two. Converting a domain seat into a legacy seat is fallible.
Projection entry points reject larger states and unsupported report facts, including forced
movement victims, instead of truncating them. Legacy room, score, reconnect and publication
boundaries reject additional seats before mutation. No new network or replay identity is
advertised by this headless profile. See the [release record](../releases.md) for acceptance.

## Immutable room configuration

`RoomConfiguration` describes 2–4 canonical human/CPU controls. Seat One must be human;
Easy/Standard difficulty is present exactly when at least one CPU is configured. Accessors
expose only actual seats, even though private fixed-capacity storage reserves four entries.
`BoardSize` recognizes 11×9, 13×11 and 15×13; current generation explicitly accepts only
15×13. These domain values do not change live room creation or transport contracts.
