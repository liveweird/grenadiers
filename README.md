# Grenadiers

A browser-based multiplayer game inspired by classic Bomberman/Dynablaster.
The initial playable profile supports two human players. Bots and larger matches follow later.

Wall Pass is published as the eighteenth requested item in prerelease
[v0.3.43](https://github.com/liveweird/grenadiers/releases/tag/v0.3.43). Published source
`c086f719ea470b9fe24963adef84b5e3bdf3e17a`, tree
`d74fad8a1c870f2834fca48f80ea4e38f0c512d9`, is tagged by annotated object
`c8395fd7dcc5701842644967802e61c76211931c`. Candidate push/PR runs `40051959641` and
`40051963999`, master run `40053637247`, and tag run `40055400664` pass all ten jobs, 55 browser
cases and all 262,144 settings masks in each Rust profile on their first attempts. Localhost serves
the exact accepted image `sha256:577abb32958e45912f35a7f6ae68dd624deb6515d2b4d55519942c44436a2a01`
from container `d7f68cc89bfc87d06c202fbedae78c1892f54ea3ccdc6571fee0d1c5ea2f4240`;
health, readiness and index return 200 and the served script SHA-256 is
`61d6722543d93559ad980b2f6577897ddbdfcc80b4137d598d7aecf75c6ea637`.

The 110 native cases remain attributed to source `11d0dc7e39e8e2cafcf70c1548be339f52c740ab`
and final manifest `3ac5e05ccee94c7716376ca9383c75acc924b772447499c6a0dbd857f68ec15c`.
Reviewed artifact equality connects corrected source `5dbcd58e920f852a8036c4bf741e26517793b072`
to the published source through eight documentation-only changes; it does not claim unchanged
Docker inputs, a new Rust compilation or new native execution. Delivery binding
`40578e13d8a44a3cdb22bc222ac71eedd0005b08f3fb5f0748d07f7e333cf5d1` and its independent review
are clear. The v0.3.42 image remains retained as the rollback parent. Eighteen of 22 requested
items are published; Kick is the v0.3.44 source-only candidate, with fresh gates, 56+56 native
acceptance and CI still pending. Punch, Bracelet and Oil Barrel follow before ART-ATLAS.

The **two-player arena match is playable**. Create a room, share its invitation,
join from another browser, and both choose Ready. Select the courtyard to move with arrows
or W/A/S/D and plant bombs with Space. Rust owns the 60 Hz simulation; PixiJS draws public
20 Hz snapshots while React/Mantine provides the room controls, scores, phase timers, results, and rematch.

The [retro pixel-art arena](docs/game/pixel-art.md) adds original weathered terrain, directional
characters, animated grenades/flames and distinct item symbols, with reduced-motion support.

Default rooms allow turns within a quarter tile of a corridor centerline, including natural
wall stops. The creator can adjust starting speed, turning tolerance, wins needed and fuse
time in the lobby's shared [gameplay settings](docs/game/settings.md). New rooms enable the
[Heart pickup](docs/game/heart.md), which absorbs one lethal blast. Disable it or choose the
Classic settings preset for the original loadout. Enable the optional [Shield](docs/game/shield.md)
for five seconds of immunity; its timer refreshes on collection. The optional
[Golden Flame](docs/game/golden-flame.md) gives newly planted bombs maximum range for ten seconds. Optional
[Short Fuse](docs/game/short-fuse.md) halves newly planted bomb fuses for ten seconds. The opt-in
[Short Flame curse](docs/game/short-flame.md) limits new bombs to one adjacent cell for ten seconds. The opt-in
[Long Fuse curse](docs/game/long-fuse.md) doubles newly planted bomb fuses for ten seconds. Optional
[Reverse Controls](docs/game/reverse-controls.md) reverses held movement directions for ten seconds. The opt-in [Diarrhea curse](docs/game/diarrhea.md) automatically drops ordinary bombs when placement is possible for ten seconds. [Slow foot](docs/game/slow-foot.md) halves movement speed for ten seconds per independently timed stack. [Kick](docs/game/kick.md) is permanent equipment that pushes stationary Timed and Remote bombs on a fixed cadence while preserving their fuse and ownership. [Wall Pass](docs/game/wall-pass.md) is permanent equipment for crossing breakable blocks while hard walls, bombs and hazards remain effective. [Remote Bomb](docs/game/remote-bomb.md) stores one manual bomb that waits for its owner's next bomb action. [Piercing Bomb](docs/game/piercing-bomb.md) stores one manual bomb whose blast crosses breakable blocks. [Power Bomb](docs/game/power-bomb.md) stores one manual bomb whose diamond-shaped blast ignores obstruction while preserving hard walls. [Landmine](docs/game/landmine.md) stores one passable contact mine with initial exit grace. [Invisibility Cloak](docs/game/invisibility-cloak.md) hides an active actor's spatial and loadout details from the opponent for ten seconds. [Stopwatch](docs/game/stopwatch.md) freezes the opponent's voluntary actions and pauses Timed bombs for three seconds. [Invisible Bombs](docs/game/invisible-bombs.md) hides every live bomb and its cues from the afflicted player for ten seconds. [Teleporter](docs/game/teleporter.md) consumes an immediate pickup and swaps both living players only when both destinations are safe. Gameplay v10 requires reloading older browser
clients after an upgrade; see the [compatibility note](docs/game/protocol-v10.md).
Reviewed pre-collection examples show Wall Pass in [English desktop](docs/screenshots/wall-pass-desktop.png)
and [Polish 320px reduced-motion](docs/screenshots/wall-pass-mobile-pl.png) presentation. The
published Teleporter examples remain available as [English desktop](docs/screenshots/teleporter-desktop.png)
and [Polish 320px reduced-motion](docs/screenshots/teleporter-mobile-pl.png) captures.

Use **Keyboard controls** below the arena to remap movement and bombs or restore defaults. Preferences
stay in this browser. Opening settings, losing focus, or hiding the tab releases held movement;
returning requires a fresh key press. See the [keyboard guide](docs/game/controls.md).

Optional [sound cues](docs/game/audio.md) start muted. Select **Enable sound** below the arena;
returning from a hidden tab requires **Resume sound**. Essential game information stays visual.

Reach the configured win target (three by default) to take the match. Draws award no score; automatic countdowns reset the
world between rounds, and both players can consent to a rematch. A dropped tab can restore its
reserved seat for 600 server ticks using a private in-memory capability; a connected takeover
fences the old tab. The match requires a keyboard and keeps active state and capabilities in
one server process and browser memory. Matchmaking, durable stats, bots, and larger matches
follow later. Read the [browser guide](docs/game/browser.md), [game specification](docs/game/README.md),
and [implementation/release plan](docs/game/implementation-plan.md) for behavior and scope.

## Stack and layout

| Directory | Responsibility |
| --- | --- |
| `core/` | Rust wire types, deterministic game rules, and room lifecycle, independent of I/O |
| `server/` | Axum/Tokio HTTP and WebSocket adapters, tracing, optional SQLx readiness |
| `web/` | React, TypeScript, Vite, Mantine, PixiJS arena, EN/PL localization |
| `api/` | Authoritative OpenAPI and realtime JSON Schema contracts |
| `api-guidelines/` | Stable API rules and Spectral configuration |
| `e2e/` | Standalone Playwright blackbox tests and matching scenario documents |
| `docs/` | Architecture decisions, conventions, testing, and operational guidance |
| `k8s/` | Single-replica local Kubernetes deployment and Kustomize composition |
| `scripts/` | Local gates shared with CI |

See [architecture](docs/architecture.md), [Flow equivalents](docs/decisions/0001-stack.md),
[testing](docs/testing.md), [deployment](docs/deployment.md), and [contributing](CONTRIBUTING.md).
Product versioning and candidate checks are documented in [releases](docs/releases.md).
The [headless movement API](docs/game/movement.md) and [combat phases](docs/game/combat.md)
document the deterministic domain; the arena uses
[PixiJS alongside React](docs/decisions/0003-arena-rendering.md).

## Prerequisites

Use `mise install` for Node **24.21.0** and Rust **1.99.1**, then `mise exec -- <command>`
or activate mise in your shell. `rust-toolchain.toml` also selects the Rust toolchain.
Docker is optional for local development and required for the complete Compose stack.

Cargo and npm dependencies are resolved in repository lockfiles. Normal builds use locked
dependencies; review lockfile changes together with dependency updates.
See [verification status](docs/verification.md) for checks actually performed.

```sh
npm --prefix web ci
npm --prefix e2e ci
cargo fetch --locked
```

## Run locally

Run these in separate terminals:

```sh
cargo run --locked -p grenadiers-server
npm --prefix web run dev
```

Open <http://localhost:5177>. Vite proxies `/api`, including WebSockets, to `127.0.0.1:8085`.
The page's Connect/Ping controls verify connectivity; they do not join a match or reserve a player slot.

For a single-origin production-style local stack:

```sh
docker compose up --build
```

Open <http://localhost:8085>. The server serves the built SPA and API. Stop with
`docker compose down`; add `--volumes` only when you intend to delete local database data.

For local Kubernetes, use the manifests under `k8s/` after building and loading the application
image. See [deployment instructions](docs/deployment.md) and the
[verification record](docs/verification.md) for runtime results.

## Optional PostgreSQL

The scaffold needs no database. Start one only when working on durable persistence:

```sh
docker compose --profile persistence up -d postgres
export DATABASE_URL=postgres://grenadiers:grenadiers_local_only@127.0.0.1:5436/grenadiers
cargo run --locked -p grenadiers-server
```

Without `DATABASE_URL`, readiness has no database dependency. With it set, database failure makes
`/api/v1/ready` return 503; `/api/v1/health` remains a liveness probe. There are no tables or
migrations yet. PostgreSQL is intended for users/results, not the per-tick game loop.

## Checks

```sh
scripts/gates.sh all           # Rust, contracts, web, and E2E static gates
scripts/gates.sh server        # rustfmt, Clippy, tests, build (locked)
scripts/gates.sh web           # contract drift, lint, dead code, coverage, build
scripts/gates.sh e2e           # lint, typecheck, dead code, scenario parity
scripts/gates.sh browser       # default Playwright scenarios; requires the composed stack
scripts/gates.sh k8s           # Kustomize render + strict Kubernetes schema validation
scripts/gates.sh security      # cargo-deny and npm dependency audits
scripts/gates.sh coverage      # cargo-llvm-cov line coverage floor
```

Generate frontend contracts with `npm --prefix web run gen:api` and
`npm --prefix web run gen:realtime`; gameplay controls use `npm --prefix web run gen:gameplay`.
Commit generated files with contract changes. The [gameplay protocol](docs/game/protocol.md)
defines authentication, room controls, and public snapshot/event messages. Session issuance,
authenticated sockets, room admission, authoritative rounds, and browser controls are implemented.
Reconnect acceptance runs separately with
`E2E_MATCH_FIXTURE=arena-v1-matches npm --prefix e2e run test:reconnect`.
Coverage tooling installation and test conventions are in [docs/testing.md](docs/testing.md).

## Configuration

`.env.example` documents development defaults. The Rust process reads environment variables;
it does not automatically load `.env` files. Compose reads `.env` for interpolation.
Host ports **8085**, **5177**, and **5436** avoid Flow's defaults.
See [security and configuration](docs/security.md) before exposing the scaffold beyond localhost.

## Contributor guidance

The original `AGENTS.md` still describes the empty workspace. It is preserved under the
initial instruction not to modify an existing file; use `CONTRIBUTING.md` and `docs/` for
current commands and conventions.
