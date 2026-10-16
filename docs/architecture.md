# Architecture

The implemented vertical slice is diagnostic transport: HTTP liveness/readiness and a
WebSocket welcome/ping/pong exchange. The React shell exercises it. There is no game session,
player allocation, authentication, combat, or persistence model yet. A separate headless
world/movement model now lives in `core::game`; the diagnostic routes do not run it.

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

For the first playable version, one in-process owner will hold each match and advance a
server-controlled fixed tick. Human and future bot adapters will submit the same commands
through bounded queues; clients will not submit authoritative positions, damage, or outcomes.
These are architectural constraints for the next phase, not implemented gameplay.

Start with two players per match. Transport sockets are not player seats; the diagnostic
welcome's `maxPlayers: 2` describes the planned initial match capacity. Support for more
players and AI must be designed around match configuration rather than global connections.

Active state belongs in memory. Process restarts will end active matches until recovery is
explicitly designed. PostgreSQL can later store identities, completed results, or replays;
there are no dummy tables or empty migrations in this scaffold. Adopt append-only SQLx
migrations with the first real schema; do not mutate applied migration files.

Redis is unnecessary initially. Multiple server replicas would require match routing,
single-owner guarantees, reconnect semantics, and deployment draining. Do not add replicas
or shared mutable state before those decisions are documented and tested.

## Deferred decisions

M1 fixes the integer 60 Hz movement model in [ADR 0002](decisions/0002-deterministic-movement.md).
The [headless API](game/movement.md) covers its state, collision, and deterministic input boundary.
[ADR 0003](decisions/0003-arena-rendering.md) selects PixiJS for the future arena and React for
menus/HUD. No renderer dependency is installed yet.

Map generation, combat, lag compensation, room admission, authentication, and matchmaking
still need implementation decisions at their planned milestones. Add focused ADRs before
dependent work; preserve the specified lifecycle and gameplay rules.


The [draft game specification](game/README.md) proposes a two-player ruleset and catalogs
classic mechanics and optional extensions. Its values and policies are discussion proposals;
they do not change the implemented scaffold or settle the deferred architecture decisions above.
The [implementation plan](game/implementation-plan.md) identifies when each dependent decision
must be recorded and how the two-player baseline will be delivered and verified.
