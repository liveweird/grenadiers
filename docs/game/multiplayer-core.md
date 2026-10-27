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

Rules versions 1–5 accept only two actors. `RulesConfig::multiplayer_v6(settings)` is a
headless construction profile. With three/four actors it currently requires an empty
optional-item mask; basic bomb capacity, flame range and speed pickups remain available.
Unsupported items and world facts are rejected rather than running pair-only abilities.

## Per-round arena and actions

Arena metadata describes one round's actual spawn cells. Two-player opposite-corner rules
are unchanged; three/four-player spawns must be distinct interior corners with clear inward
escape cells. `RoundAssignment` rotates seat assignment across the supplied cells and action
priority across participants. `spawn_for` returns `None` for a seat absent from that arena.

A three-player series must construct its arena metadata anew for each round to rotate the
unused fourth corner. Reusing the same three spawn cells does not perform that rotation.
Generation owns this choice; the state constructor does not rewrite caller terrain or spawns.
Historical generators retain their two-player behavior.

All input seats are validated before tick advancement or mutation. Simultaneous placement
at a shared cell rejects every contender; independent placements use canonical ordering.
Pickup ties use the rotated participant priority. Zero survivors draw, one wins, and multiple
survivors continue until the configured timeout. Actor-sized traces and event audiences
contain only actual participants; legacy `EventAudience::Both` still means One and Two.

## Archived compatibility

Historical wire seats remain One/Two. Converting a domain seat into a legacy seat is fallible.
Projection entry points reject larger states and unsupported report facts, including forced
movement victims, instead of truncating them. Legacy room, score, reconnect and publication
boundaries reject additional seats before mutation. No new network or replay identity is
advertised by this headless profile. See the [release record](../releases.md) for acceptance.
