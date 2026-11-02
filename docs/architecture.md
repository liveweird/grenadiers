# Architecture

The server provides HTTP liveness/readiness, diagnostic WebSockets, anonymous credentials,
and authenticated authoritative two-to-four-seat matches with at least one human controller
and a configurable win target (three by default). React/Mantine provides room admission and
the HUD; Three.js and React Three Fiber render public game snapshots.
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

One in-process task owns each room and advances a server-controlled fixed tick. Human inputs and
server-owned CPU intentions arrive through the same bounded command boundary.
The pure [CPU contracts](game/cpu-contracts.md) define controller identity, restricted
observations, ordinary intentions and bounded decision state for that integration.
Clients do not submit authoritative positions, damage, or outcomes.
The owner executes bounded logical intentions; public snapshots carry the accepted results.

Matches configure two to four seats and require at least one human controller. Transport sockets
are not player seats; the diagnostic v2 welcome's `maxPlayers: 4` describes the live match-capacity
ceiling, not a connection limit. Live two-to-four-seat matches use CPU50 on Easy/Standard and
CPU51 on Hard, rules10 physics and mob navigation3; cloud, lair and spike-pit settings may be
enabled or disabled per room. These policies add the typed Teleporter recovery to CPU44/45's
unchanged ordinary result, memory and RNG behavior. Confirmed-origin continuation still delegates
to CPU44/45, and older profiles retain their historical policies and rules. The board
choice is immutable, defaults to 15×13 and is documented in [board geometry](game/board-sizes.md).
Match configuration, rather than global connections, determines the participant roster.

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
[ADR 0029](decisions/0029-three-billboard-arena.md) selects Three.js/React Three Fiber for the
production arena, retaining Pixi only for isolated artwork diagnostics, and React for
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
The [implementation plan](game/implementation-plan.md) defines delivery tracks and release
verification; the [backlog](../BACKLOG.md) owns sequencing and work status.

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

The active `roster_runtime` server adapter owns real per-round physics, assignment-aware
generation, epoch-fenced input, settled reconnect grace, and checked v20 publication. Result
physics freezes while the global lifecycle clock and reservation deadlines advance.

The active v20 client/panel shares the Three.js presentation boundary, validates v20 messages, keeps
the resume capability only in tab memory, and renders scores, recovery, grace, and rematch from
authoritative facts. Production routing uses v20 on `/api/v1/game/ws`; versions 1 through 19 are
rejected before credential claim. Gameplay v7–v11 added
the expanded item inventory and recipient-private projections; the live contract is
[protocol v20](game/protocol-v20.md). See [reconnect browser](game/reconnect-browser.md).

The active [bounded match owner](game/match-owner.md) adds generation-fenced admission,
scheduled departure batches, absolute deadlines, retention, and sticky shutdown. Its closing
watch value retains the final checked publication. On shutdown, member sockets send it before
closing, and the process waits up to five seconds for all socket permits and room cleanup.

Classic-v3 widens turn assistance to 256/1,024 tile units. Gameplay v5 activates this rules identity
without extending the archived closed v4 contract; [migration](game/protocol-v5.md) preserves
classic-v1/v2 movement and recorded-profile replay.

[ADR 0026](decisions/0026-solo-controllers.md) defines the implemented private
[solo mode](game/single-player.md), server-owned controller, bounded planning and committed
escape execution. Solo replay adds header provenance while replaying ordinary accepted inputs.
