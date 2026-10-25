# Architecture

The server provides HTTP liveness/readiness, diagnostic WebSockets, anonymous credentials,
and authenticated authoritative two-seat matches with a configurable win target (three by
default). React/Mantine provides room admission and
the HUD; PixiJS renders public game snapshots.
A separate deterministic simulation in `core::game` implements movement, owned bombs,
flames, damage, and one-round outcomes. Room owners advance that simulation at a fixed
server-controlled tick.

## Boundaries

`core` holds diagnostic transport types and pure game rules. `server` owns all I/O:
Axum routes, WebSocket lifecycles, configuration, optional PostgreSQL connectivity, and tracing.
`web` handles presentation and input. `api` is the source of truth for wire formats.
Clients must validate untrusted server messages as well as servers validating client messages.

Production-style development uses one origin: Axum serves `web/dist` and `/api/v1/*`.
Vite's development proxy preserves the browser Origin and upgrades WebSockets. Unknown API
paths return problem JSON, never the SPA fallback. Health means the process is alive;
readiness checks optional PostgreSQL only when configured.

## Match-state decision

One in-process task owns each room and advances a server-controlled fixed tick. Human inputs
arrive through bounded queues; future bot adapters must use the same command boundary.
Clients do not submit authoritative positions, damage, or outcomes.
The owner executes bounded logical intentions; public snapshots carry the accepted results.

Start with two players per match. Transport sockets are not player seats; the diagnostic
welcome's `maxPlayers: 2` describes the initial match capacity. Support for more
players and AI must be designed around match configuration rather than global connections.

Active state belongs in memory. Process restarts will end active matches until recovery is
explicitly designed. PostgreSQL can later store identities, completed results, or replays;
there are no dummy tables or empty migrations in this scaffold. Adopt append-only SQLx
migrations with the first real schema; do not mutate applied migration files.

Redis is unnecessary initially. Multiple server replicas would require match routing,
cross-process capability routing, single-owner guarantees, and deployment draining. Do not add replicas
or shared mutable state before those decisions are documented and tested.

## Implemented domain and presentation decisions

M1 fixes the integer 60 Hz movement model in [ADR 0002](decisions/0002-deterministic-movement.md).
The [headless API](game/movement.md) covers its state, collision, and deterministic input boundary.
[ADR 0003](decisions/0003-arena-rendering.md) selects exact PixiJS 8.22.0 for the arena and React for
menus/HUD. [Browser behavior](game/browser.md) documents input, interpolation, and teardown.

[Combat phases](game/combat.md) and ADRs 0004–0006 define owned placement, immutable explosion
occupancy, and round finality. [ADR 0007](decisions/0007-public-state-events.md) defines the
explicit public projection and phase events, completing M2 before the server adapter.

[ADR 0008](decisions/0008-gameplay-protocol.md) and the [gameplay contract](game/protocol.md)
define anonymous first-frame authentication, control/lifecycle messages, and bounded public
snapshot/event wire shapes. [ADR 0013](decisions/0013-gameplay-v2-and-room-profiles.md)
adds gameplay-v2 exposed items/events and immutable construction/publication profiles. Explicit fallible adapters check public domain state and bounded
report windows before wire publication. `POST /api/v1/game/session` now issues bounded,
short-lived anonymous credentials using OS randomness. The gameplay WebSocket handler
atomically claims one owner and enforces bounded connection policy. Diagnostics uses separate
connection and upgrade-rate budgets, so diagnostic traffic cannot consume gameplay admission
capacity. A bounded room registry routes code-based admission to one owner task per room.

[ADR 0009](decisions/0009-room-lifecycle.md) keeps the implemented headless two-seat
[lifecycle](game/rooms.md) in `core::room`. Credentials, discovery, binding generations,
publication, and the asynchronous room owner stay outside that pure state machine. The server
implements discovery, generation-bound membership, departure cleanup, countdown, and running
publications. [ADR 0010](decisions/0010-authoritative-room-loop.md) fixes scheduling, input
budgets, result ordering, and retention.

[ADR 0011](decisions/0011-deterministic-arena-generation.md) defines the implemented headless
[generator](game/generation.md). [ADR 0014](decisions/0014-private-round-seeds-and-live-generation.md)
activates generated classic-v2 rooms with private round seeds and retained records. Reconnect is
implemented for one process lifetime; lag compensation and matchmaking still need their
planned milestones. Add focused ADRs before
dependent work; preserve the specified lifecycle and gameplay rules.


The [game specification](game/README.md) is the working two-player baseline and catalogs
classic mechanics and optional extensions. Its initial profile drives the simulation; tuning
remains subject to playtesting. Optional extensions do not become baseline requirements.
The [implementation plan](game/implementation-plan.md) identifies when each dependent decision
must be recorded and how the two-player baseline will be delivered and verified.

[ADR 0012](decisions/0012-pickup-phases-and-upgrades.md) defines the implemented typed
[pickup layers](game/pickups.md) with classic-v2 collection/upgrades and explicit round-number spawn/tie assignment.

[ADR 0015](decisions/0015-match-series-and-staged-progression.md) separates implemented
[headless match-series facts](game/match-series.md) and the pure [automatic lifecycle](game/match-lifecycle.md)
from transport integration. Shared room settings now select one to nine wins; Classic defaults
to three. Settings remain immutable through automatic rounds.

[ADR 0016](decisions/0016-dormant-gameplay-v3-contract.md) records the earlier namespaced
[gameplay-v3 contract](game/protocol-v3.md) separately from archived v2. Gameplay v4 extends
that physical contract with resumable admissions; generated types never fabricate scores.

V4 checked room projection derives facts directly from `MatchLifecycle`; its public validator
rejects semantically impossible bounded DTOs before server adapters publish them.

The active `match_runtime` server adapter owns real per-round physics, assignment-aware
generation, epoch-fenced input, settled reconnect grace, and checked v11 publication. Result
physics freezes while the global lifecycle clock and reservation deadlines advance.

The active v11 client/panel shares the Pixi presentation boundary, validates v11 messages, keeps
the resume capability only in tab memory, and renders scores, recovery, grace, and rematch from
authoritative facts. Production routing uses v11 on the existing endpoint. Gameplay v7–v11 added
the expanded item inventory and recipient-private projections; the live contract is
[protocol v11](game/protocol-v11.md). See [reconnect browser](game/reconnect-browser.md).

The active [bounded match owner](game/match-owner.md) adds generation-fenced admission,
scheduled departure batches, absolute deadlines, retention, and sticky shutdown. Its closing
watch value retains the final checked publication. On shutdown, member sockets send it before
closing, and the process waits up to five seconds for all socket permits and room cleanup.

Classic-v3 widens turn assistance to 256/1,024 tile units. Gameplay v5 activates this rules identity
without extending the archived closed v4 contract; [migration](game/protocol-v5.md) preserves
classic-v1/v2 movement and recorded-profile replay.
