# Multiplayer and configurable arena design

This specifies the accepted expansion. Current implemented guarantees remain in the
[room](rooms.md), [solo](single-player.md) and [protocol](protocol.md) guides. Delivery order
and outstanding milestones live only in [BACKLOG.md](../../BACKLOG.md#multiplayer-and-arenas).

## Participants and authority

Support two, three or four participants, with at least one human. The creator is human seat
One; remaining configured seats are human or CPU. CPU difficulty is room-wide Easy/Standard.
Player count and controllers are fixed at creation and retained through reconnect/rematch.
Human admission fills configured human vacancies and never claims a CPU slot. This expansion
does not add spectators, all-CPU rooms, live replacements or automatic bot substitution.

Keep original creator settings authority without transferring it after departure. Connected or
reserved humans retain a room; CPU occupancy alone cannot. An individual permanent departure
removes that participant while eligible others continue. Preserve physics/scoring precedence,
reservation expiry and already-final results when settling departures.

## Rules and fairness

Zero living actors produce a draw, one wins, and multiple survivors continue until the existing
timeout. Use four corner spawn regions and rotate assignments by actual round number. With
three participants, rotate the omitted corner so no seat permanently receives one position.
Multiplayer generation must reserve fair exits and use symmetric terrain/item orbits while
historical two-player generation remains replayable unchanged.

Resolve simultaneous actions against a common pre-mutation view with deterministic rotating
priority. Extend Bracelet contacts and blockers to explicit attacker/victim pairs. Stopwatch
freezes an actor when another living actor holds it; multiple holders can freeze one another.
Teleporter swaps its collector with one randomly selected living enemy, preserving safe-swap
and consumption rules. Its replayable selection stream is independent of arena generation and
CPU randomness. Other weapons, damage and pickup rules retain their ordinary meaning.

## Compatibility and bounded execution

Separate simulation seat identity from archived wire enums. Historical schemas, projections,
generators and replay readers must keep their original closed shapes and reject unsupported
rosters/dimensions explicitly. A new live gameplay contract and replay identity must describe
the roster, per-CPU provenance, generation and simulation randomness before activation.

Validate the complete state before recipient filtering. Prepare every recipient frame before
publishing or flushing events. Prove the worst-case frame/event bounds for the larger roster;
never truncate reports to fit the existing envelope. Review advertised diagnostic capacities
and their version contract as well as gameplay transport.

Each CPU has its own memory, RNG, pending request and execution state. Keep the shared worker
pool/queue and per-job work bounds; process controllers fairly without blocking room ticks.
Recheck every relevant rival in tactical proofs, including third-party blockers and hazards.

## Appearance and board dimensions

Preserve the existing orange/white and blue uniforms. Players three/four use distinct
forest-green/cream/silver and plum-purple/cream/gold variants with different headgear. Keep
stable anchors, readable HUD identity and the single [artist source](../../assets/art/README.md).

Activate multiplayer on the existing 15×13 board before exposing size selection. Then introduce
checked dimensions, bounded supported sizes and dimension-aware indexing, generation, spawn
reservations, blasts, Punch wrapping, CPU routing and rendering. Preserve 15×13 as the default.
Select the initial size allowlist through frame-bound and CPU measurements; arbitrary unbounded
dimensions are outside this design.

## Acceptance

Preserve two-player gameplay/replay traces. Cover three/four-human, one-human/three-CPU and mixed
lineups, both CPU levels, simultaneous actions/deaths/departures, no-human cleanup, reconnect,
rematch, Teleporter replay and privacy for every recipient. Verify each supported board size
with every supported player count, legal spawn escapes, edge behavior, bounded planning and
desktop/mobile renderer lifecycle. Release evidence belongs in release/history records.
